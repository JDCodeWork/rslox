use super::chunk::{Chunk, OpCode};

#[cfg(feature = "dbg")]
use super::dbg::{dbg_stack, disasm_instr};

use crate::{compiler::Compiler, values::Value};

#[derive(Debug)]
pub enum ExecErr {
    CompileErr,
    RuntimeErr,
}

type ExecResult = Result<(), ExecErr>;

pub struct VM {
    pub chunk: Chunk,
    pub stack: Vec<Value>,
    pub ip: usize,
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

enum CompareOp {
    Equal,
    Greater,
    Less,
}

impl VM {
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
        };

        vm.run()
    }

    fn run(&mut self) -> ExecResult {
        loop {
            // region: Debugging output (--features dbg)
            #[cfg(feature = "dbg")]
            {
                dbg_stack(&self.stack);
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
                OpCode::Add => self.binary_op(BinaryOp::Add),
                OpCode::Sub => self.binary_op(BinaryOp::Sub),
                OpCode::Mul => self.binary_op(BinaryOp::Mul),
                OpCode::Div => self.binary_op(BinaryOp::Div),
                OpCode::Return => return Ok(()),
                // Should never happen
                OpCode::_COUNT => return Err(ExecErr::CompileErr),
            }?
        }
    }

    fn constant(&mut self) -> ExecResult {
        let const_ = self.read_byte();
        let value = self.chunk.constants[const_ as usize];

        self.stack.push(value);

        Ok(())
    }

    fn literal(&mut self, value: Value) -> ExecResult {
        self.stack.push(value);

        Ok(())
    }

    fn compare(&mut self, op: CompareOp) -> ExecResult {
        // Stack underflow
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

        let res = match op {
            CompareOp::Equal => a == b,
            CompareOp::Greater => a > b,
            CompareOp::Less => a < b,
        };

        self.stack.push(Value::Boolean(res));

        Ok(())
    }

    fn not(&mut self) -> ExecResult {
        // Stack underflow

        if let Some(value) = self.stack.last_mut() {
            *value = Value::Boolean(value.is_falsey());
        }

        Ok(())
    }

    fn negate(&mut self) -> ExecResult {
        // Stack underflow

        if let Some(Value::Number(value)) = self.stack.last_mut() {
            *value = -(*value);
        }

        Ok(())
    }

    fn binary_op(&mut self, op: BinaryOp) -> ExecResult {
        // Stack underflow

        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

        // TODO: Ensure that both values are the same type
        match op {
            BinaryOp::Add => self.stack.push(a + b),
            BinaryOp::Sub => self.stack.push(a - b),
            BinaryOp::Div => self.stack.push(a / b),
            BinaryOp::Mul => self.stack.push(a * b),
        };

        Ok(())
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;

        byte
    }
}
