use std::collections::HashMap;

use super::chunk::{Chunk, OpCode};

#[cfg(feature = "dbg")]
use super::dbg::{dbg_mem, disasm_instr};

use crate::{
    compiler::Compiler,
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
}

pub struct VM<'a> {
    pub chunk: Chunk,
    pub stack: Vec<Value>,
    pub heap: Vec<Object>,
    pub src: &'a str,
    pub ip: usize,

    pub strings: Interner<ObjRef>,
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
                OpCode::Return => return Ok(()),
                // Should never happen
                OpCode::_COUNT => return Err(ExecErr::CompileErr),
            }?
        }
    }

    fn constant(&mut self) -> ExecResult {
        let const_ = self.read_byte();
        let value = self.make_value(const_)?;

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
            return Err(ExecErr::RuntimeErr);
        };

        self.stack.push(value);

        Ok(())
    }

    fn operands(&mut self) -> Result<(Value, Value), ExecErr> {
        // TODO: Stack underflow

        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

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

    fn make_value(&mut self, const_: u8) -> Result<Value, ExecErr> {
        let constant = self.chunk.constants[const_ as usize].clone();

        let value = match constant {
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

    #[inline]
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;

        byte
    }
}
