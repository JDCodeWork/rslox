use crate::errors::MarshalError;

pub type Byte = u8;

#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Return,
    _COUNT,
}

pub struct Chunk {
    pub code: Vec<Byte>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn write(&mut self, byte: Byte) {
        self.code.push(byte);
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
    }
}

impl TryFrom<Byte> for OpCode {
    type Error = MarshalError;

    fn try_from(value: Byte) -> Result<Self, Self::Error> {
        if value > (OpCode::_COUNT as u8) - 1 {
            return Err(MarshalError::InvalidBytecode);
        }

        let opcode = unsafe { core::mem::transmute::<Byte, Self>(value) };
        Ok(opcode)
    }
}
