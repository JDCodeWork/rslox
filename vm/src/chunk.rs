pub type Byte = u8;

pub type Value = f64;

#[derive(Debug)]
pub enum MarshalError {
    InvalidBytecode,
    InvalidPrecedence,
}

pub struct RleArr {
    pub base_ln: usize,
    pub curr_ln: usize,
    pub deltas: Vec<Byte>,
    pub counts: Vec<Byte>,
}

#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Constant,
    Negate,
    Add,
    Sub,
    Mul,
    Div,
    Return,
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
    pub fn write<B: Into<Byte>>(&mut self, byte: B, ln: usize) {
        self.code.push(byte.into());

        if self.rles.curr_ln == ln {
            self.rles.incr_count();
        } else if self.rles.curr_ln + 1 < ln {
            self.rles.add_rle();

            let delta_count = ln - self.rles.curr_ln;
            for _ in 0..delta_count {
                self.rles.incr_delta();
            }
        } else {
            self.rles.add_rle();
        }
    }

    /// Add a value to constants and returns the position
    pub fn add_const(&mut self, value: Value) -> usize {
        self.constants.push(value);

        self.constants.len() - 1
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

impl RleArr {
    pub fn new() -> Self {
        Self {
            base_ln: 0,
            curr_ln: 0,
            deltas: Vec::new(),
            counts: Vec::new(),
        }
    }

    /// Increases the value of the last element in the counts array. Use when the OpCode belongs to the same line as the previous one
    pub fn incr_count(&mut self) {
        if let Some(count) = self.counts.last_mut() {
            *count += 1;
        }
    }

    #[allow(dead_code)]
    /// Increase the value of the last element in the deltas array. Mainly used for instructions whose delta is greater than 1
    pub fn incr_delta(&mut self) {
        if let Some(delta) = self.deltas.last_mut() {
            *delta += 1;
        } else {
            self.add_rle();
        }

        self.curr_ln += 1;
    }

    /// Add new delta with a vale of 1 and a count with a value of 0. Use when starting a new line.
    pub fn add_rle(&mut self) {
        self.deltas.push(1);
        self.counts.push(1);

        self.curr_ln += 1;
    }

    pub fn get_ln(&self, offset: usize) -> usize {
        let mut delta_acc = 0;
        let mut count_acc = 0;

        for (delta, count) in self.deltas.iter().zip(self.counts.iter()) {
            delta_acc += *delta as usize;
            count_acc += *count as usize;

            if offset < count_acc {
                return self.base_ln + delta_acc;
            }
        }

        self.base_ln
    }
}

impl core::fmt::Display for MarshalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarshalError::InvalidBytecode => f.write_str("Invalid bytecode"),
            MarshalError::InvalidPrecedence => f.write_str("Invalid precedence"),
        }
    }
}

impl core::error::Error for MarshalError {}
