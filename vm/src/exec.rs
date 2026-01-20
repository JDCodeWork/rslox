use super::chunk::{Chunk, OpCode, Value};
use super::dbg::{dbg_stack, disasm_instr};

use crate::compiler::Compiler;

pub enum InterpretResult {
    Ok,
    CompileErr,
    RuntimeErr,
}

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

impl VM {
    pub fn interpret(source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();
        let mut c = Compiler::new(source, &mut chunk);

        if !c.compile() {
            return InterpretResult::CompileErr;
        }

        let mut vm = VM {
            chunk,
            ip: 0,
            stack: Vec::new(),
        };

        vm.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            // region: Debugging output (--feature dbg)
            dbg_stack(&self.stack);
            disasm_instr(self.ip, &self.chunk);
            // endregion: Debugging output (--feature dbg)

            let instr = self.read_byte();

            let Ok(opcode) = OpCode::try_from(instr) else {
                // MarshallErr
                return InterpretResult::CompileErr;
            };

            match opcode {
                OpCode::Constant => self.constant(),
                OpCode::Negate => self.negate(),
                OpCode::Add => self.binary_op(BinaryOp::Add),
                OpCode::Sub => self.binary_op(BinaryOp::Sub),
                OpCode::Mul => self.binary_op(BinaryOp::Mul),
                OpCode::Div => self.binary_op(BinaryOp::Div),
                OpCode::Return => return InterpretResult::Ok,
                // Should never happen
                OpCode::_COUNT => return InterpretResult::CompileErr,
            };
        }
    }

    fn constant(&mut self) {
        let const_ = self.read_byte();
        let value = self.chunk.constants[const_ as usize];

        self.stack.push(value);
    }

    fn negate(&mut self) {
        if let Some(value) = self.stack.last_mut() {
            *value = -(*value);
        }
    }

    fn binary_op(&mut self, op: BinaryOp) {
        // TODO: Implement error handling for stack underflow error
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

        match op {
            BinaryOp::Add => self.stack.push(a + b),
            BinaryOp::Sub => self.stack.push(a - b),
            BinaryOp::Div => self.stack.push(a / b),
            BinaryOp::Mul => self.stack.push(a * b),
        };
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;

        byte
    }
}
