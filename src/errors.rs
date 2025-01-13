use thiserror::Error as ThisError;

pub struct Error {
    error_type: ErrorType,
    line: Option<isize>,
}

#[derive(ThisError, Debug)]
pub enum ErrorType {
    #[error(transparent)]
    Lox(#[from] LoxError),
    #[error(transparent)]
    System(#[from] SystemError),
}

#[derive(ThisError, Debug)]
pub enum LoxError {
    #[error("Unexpected character.")]
    UnexpectedChar,
}

#[derive(ThisError, Debug)]
pub enum SystemError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("File not found in path: '{0}'")]
    FileNotFound(String),
    #[error("Invalid file extension, expected '.lox' extension")]
    InvalidFileExtension
}

impl From<LoxError> for Error {
    fn from(error: LoxError) -> Self {
        Error {
            error_type: ErrorType::Lox(error),
            line: None,
        }
    }
}

impl From<SystemError> for Error {
    fn from(error: SystemError) -> Self {
        Error {
            error_type: ErrorType::System(error),
            line: None,
        }
    }
}

impl Error {
    pub fn with_line(mut self, line: isize) -> Self {
        self.line = Some(line);

        self
    }

    pub fn report(self) -> ErrorType{
        match &self.error_type {
            ErrorType::Lox(err) => {
                if let Some(line) = self.line {
                    println!("[ line {line} ] Error: {}", err.to_string())
                } else {
                    println!("Lox error: {}", err.to_string())
                }
            },
            ErrorType::System(err) => {
                println!("System error: {}", err.to_string())
            }
        }

        self.error_type
    }
}
