#[derive(Debug)]
pub enum MarshalError {
    /// Invalid bytecode
    InvalidBytecode,
}

impl core::fmt::Display for MarshalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarshalError::InvalidBytecode => f.write_str("Invalid bytecode"),
        }
    }
}

impl core::error::Error for MarshalError {}
