use std::collections::HashMap;

use super::chunk::{Chunk, OpCode};

#[cfg(feature = "dbg")]
use super::dbg::{dbg_mem, disasm_instr};

use crate::{
    compiler::Compiler,
    scanner::Span,
    values::{ArithOp, ArithmeticError, CompareOp, Constant, ObjRef, Object, StrObj, Value},
};

#[derive(Debug)]
pub enum ExecErr {
    CompileErr,
    RuntimeErr,
}

type ExecResult = Result<(), ExecErr>;

pub struct Interner<T> {
    table: HashMap<Box<str>, T>,
}

impl<T> Interner<T> {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn get(&self, k: &str) -> Option<&T> {
        self.table.get(k)
    }

    fn insert(&mut self, k: &str, v: T) {
        self.table.insert(k.into(), v);
    }

    fn set(&mut self, k: &str, v: T) -> bool {
        if let None = self.table.insert(k.into(), v) {
            true
        } else {
            false
        }
    }

    fn delete(&mut self, k: &str) {
        self.table.remove(k);
    }
}

pub struct VM<'a> {
    pub chunk: Chunk,
    pub stack: Vec<Value>,
    pub heap: Vec<Object>,
    pub src: &'a str,
    pub ip: usize,

    pub strings: Interner<ObjRef>,
    pub globals: Interner<Value>,
}

impl<'a> VM<'a> {
    pub fn interpret(source: &str) -> ExecResult {
        let mut chunk = Chunk::new();
        let mut c = Compiler::new(source, &mut chunk);

        if !c.compile() {
            return Err(ExecErr::CompileErr);
        }

        let mut vm = VM {
            chunk,
            ip: 0,
            stack: Vec::new(),
            heap: Vec::new(),
            strings: Interner::new(),
            globals: Interner::new(),
            src: source,
        };

        vm.run()
    }

    fn run(&mut self) -> ExecResult {
        loop {
            // region: Debugging output (--features dbg)
            #[cfg(feature = "dbg")]
            {
                dbg_mem(&self.stack, &self.heap);
                disasm_instr(self.ip, &self.chunk);
            }
            // endregion: Debugging output (--features dbg)

            let instr = self.read_byte();

            let Ok(opcode) = OpCode::try_from(instr) else {
                // MarshallErr
                return Err(ExecErr::CompileErr);
            };

            match opcode {
                OpCode::Cons => self.constant(),
                OpCode::Pop => self.pop_stack().map(|_| ()),
                OpCode::Dup => self.duuplicate(),

                OpCode::Print => self.print(),
                OpCode::DefGlob => self.def_global(),
                OpCode::GetGlob => self.get_global(),
                OpCode::GetLocal => self.get_local(),
                OpCode::SetGlob => self.set_global(),
                OpCode::SetLocal => self.set_local(),

                OpCode::Jump => self.jump(),
                OpCode::JumpIfFalse => self.jump_if_false(),
                OpCode::Loop => self._loop(),

                OpCode::True => self.literal(Value::Boolean(true)),
                OpCode::False => self.literal(Value::Boolean(false)),
                OpCode::Nil => self.literal(Value::Nil),

                OpCode::Not => self.not(),
                OpCode::Eq => self.compare(CompareOp::Equal),
                OpCode::Greater => self.compare(CompareOp::Greater),
                OpCode::Less => self.compare(CompareOp::Less),

                OpCode::Neg => self.negate(),
                OpCode::Add => self.binary_op(ArithOp::Add),
                OpCode::Sub => self.binary_op(ArithOp::Sub),
                OpCode::Mul => self.binary_op(ArithOp::Mul),
                OpCode::Div => self.binary_op(ArithOp::Div),
                OpCode::Mod => self.binary_op(ArithOp::Mod),

                OpCode::Return => return Ok(()),
                // Should never happen
                OpCode::_COUNT => return Err(ExecErr::CompileErr),
            }?
        }
    }

    fn duuplicate(&mut self) -> ExecResult {
        let last_value = self.last_stack()?;
        self.stack.push(last_value);

        Ok(())
    }

    fn jump(&mut self) -> ExecResult {
        let offset = self.read_short();
        self.ip += offset as usize;

        Ok(())
    }

    fn jump_if_false(&mut self) -> ExecResult {
        let offset = self.read_short();

        if self.last_stack()?.is_falsey() {
            self.ip += offset as usize;
        }

        Ok(())
    }

    fn _loop(&mut self) -> ExecResult {
        let offset = self.read_short();
        self.ip -= offset as usize;

        Ok(())
    }

    fn set_global(&mut self) -> ExecResult {
        let span = self.read_str()?;
        let var_name = &self.src[span.start..span.end];

        let value = self.last_stack()?;
        if self.globals.set(var_name, value) {
            self.globals.delete(var_name);
            self.runtime_err(&format!("Undefined variable '{var_name}' "));
            return Err(ExecErr::RuntimeErr);
        }

        Ok(())
    }

    fn set_local(&mut self) -> ExecResult {
        let slot = self.read_byte() as usize;
        self.stack[slot] = self.last_stack()?;

        Ok(())
    }

    fn get_global(&mut self) -> ExecResult {
        let span = self.read_str()?;
        let var_name = &self.src[span.start..span.end];

        let Some(value) = self.globals.get(var_name) else {
            self.runtime_err(&format!("Undefine variable '{var_name}'"));
            return Err(ExecErr::RuntimeErr);
        };

        self.stack.push(*value);
        Ok(())
    }

    fn get_local(&mut self) -> ExecResult {
        let slot = self.read_byte() as usize;
        let value = self.stack[slot];

        self.stack.push(value);

        Ok(())
    }

    fn def_global(&mut self) -> ExecResult {
        let Span { start, end } = self.read_str()?;

        let value = self.pop_stack()?;
        self.globals.set(&self.src[start..end], value);

        Ok(())
    }

    fn print(&mut self) -> ExecResult {
        let value = self.pop_stack()?;
        if let Value::Object(id) = value {
            println!("{}", self.heap[id.0]);
        } else {
            println!("{}", value);
        }

        Ok(())
    }

    fn constant(&mut self) -> ExecResult {
        let value = self.make_value()?;
        self.stack.push(value);

        Ok(())
    }

    fn literal(&mut self, value: Value) -> ExecResult {
        self.stack.push(value);

        Ok(())
    }

    fn arith_objs(
        &mut self,
        a_ref: ObjRef,
        b_ref: ObjRef,
        op: ArithOp,
    ) -> Result<Value, ArithmeticError> {
        let a = &self.heap[a_ref.0];
        let b = &self.heap[b_ref.0];

        match (a, b, op) {
            (Object::String(a_str), Object::String(b_str), ArithOp::Add) => {
                let chars = format!("{}{}", a_str.chars, b_str.chars);

                Ok(Value::Object(self.intern_string(&chars)))
            }
            _ => Err(ArithmeticError::InvalidOperands),
        }
    }

    fn compare_objs(&mut self, a_ref: ObjRef, b_ref: ObjRef, op: CompareOp) -> bool {
        let a = &self.heap[a_ref.0];
        let b = &self.heap[b_ref.0];

        match (a, b, op) {
            (Object::String(_), Object::String(_), CompareOp::Equal) => a_ref == b_ref,
            (Object::String(a_str), Object::String(b_str), CompareOp::Less) => {
                a_str.lenght < b_str.lenght
            }
            (Object::String(a_str), Object::String(b_str), CompareOp::Greater) => {
                a_str.lenght > b_str.lenght
            }
        }
    }

    fn compare(&mut self, op: CompareOp) -> ExecResult {
        let (a, b) = self.operands()?;

        let res = match (a, b) {
            (Value::Object(a_id), Value::Object(b_id)) => self.compare_objs(a_id, b_id, op),
            _ => a.compare(b, op),
        };

        self.stack.push(Value::Boolean(res));

        Ok(())
    }

    fn not(&mut self) -> ExecResult {
        let value = self.operand()?;
        *value = Value::Boolean(value.is_falsey());

        Ok(())
    }

    fn negate(&mut self) -> ExecResult {
        if let Value::Number(value) = self.operand()? {
            *value = -(*value);
        }

        // TODO: Type error

        Ok(())
    }

    fn binary_op(&mut self, op: ArithOp) -> ExecResult {
        let (a, b) = self.operands()?;

        let res = match (a, b) {
            (Value::Object(a_ref), Value::Object(b_ref)) => self.arith_objs(a_ref, b_ref, op),
            _ => a.arithmetic(b, op),
        };

        let Ok(value) = res else {
            self.runtime_err("Invalid operands. ");
            return Err(ExecErr::RuntimeErr);
        };

        self.stack.push(value);

        Ok(())
    }

    fn operands(&mut self) -> Result<(Value, Value), ExecErr> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;

        Ok((a, b))
    }

    fn operand(&mut self) -> Result<&mut Value, ExecErr> {
        if let Some(value) = self.stack.last_mut() {
            Ok(value)
        } else {
            // Stack underflow
            Err(ExecErr::RuntimeErr)
        }
    }

    fn make_value(&mut self) -> Result<Value, ExecErr> {
        let constant = self.read_const();

        let value = match *constant {
            Constant::Number(num) => Value::Number(num),
            Constant::Boolean(b) => Value::Boolean(b),
            Constant::Nil => Value::Nil,
            Constant::String { start, end } => {
                let str_ref = self.intern_string(&self.src[start..end]);
                Value::Object(str_ref)
            }
        };

        Ok(value)
    }

    /// Allocates an object in the heap and returns `ObjRef` pointing to it
    fn allocate_obj(&mut self, object: Object) -> ObjRef {
        self.heap.push(object);
        ObjRef(self.heap.len() - 1)
    }

    fn intern_string(&mut self, s: &str) -> ObjRef {
        if let Some(str_ref) = self.strings.get(s) {
            return *str_ref;
        }

        let str_obj = Object::String(StrObj::new(s));
        let str_ref = self.allocate_obj(str_obj);

        self.strings.insert(s, str_ref);

        str_ref
    }

    fn runtime_err(&mut self, msg: &str) {
        let line = self.chunk.rles.get_ln(self.ip - 1);
        eprintln!("{msg} [line {line}] in script");
        self.stack.clear();
    }

    fn last_stack(&mut self) -> Result<Value, ExecErr> {
        if let Some(value) = self.stack.last() {
            Ok(*value)
        } else {
            self.runtime_err("Stack underflow");
            Err(ExecErr::RuntimeErr)
        }
    }

    fn pop_stack(&mut self) -> Result<Value, ExecErr> {
        if let Some(value) = self.stack.pop() {
            Ok(value)
        } else {
            self.runtime_err("Stack underflow");
            Err(ExecErr::RuntimeErr)
        }
    }

    fn read_short(&mut self) -> u16 {
        let bytes = [self.chunk.code[self.ip], self.chunk.code[self.ip + 1]];
        self.ip += 2;

        u16::from_be_bytes(bytes)
    }

    fn read_str(&mut self) -> Result<Span, ExecErr> {
        let const_ = self.read_const();

        if let Constant::String { start, end } = *const_ {
            Ok(Span { start, end })
        } else {
            // Invalid string constant
            Err(ExecErr::CompileErr)
        }
    }

    fn read_const(&mut self) -> &Constant {
        let const_ = self.read_byte();

        &self.chunk.constants[const_ as usize]
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;

        byte
    }
}
