use crate::{errors::MarshalError, rle::RleArr, value::Value};

pub type Byte = u8;

#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant,
    _COUNT,
}

pub struct Chunk {
    pub rles: RleArr,
    pub code: Vec<Byte>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            rles: RleArr::new(),
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Add a byte to code
    pub fn write<B: Into<Byte>>(&mut self, byte: B) {
        self.code.push(byte.into());
        self.rles.incr_count();
    }

    /// Add a value to constants and returns the position
    pub fn add_const(&mut self, value: Value) -> Byte {
        self.constants.push(value);

        (self.constants.len() - 1) as Byte
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

impl Into<Byte> for OpCode {
    fn into(self) -> Byte {
        self as Byte
    }
}
