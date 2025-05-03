use thiserror::Error as ThisError;

use crate::cli::alerts::Alert;

pub struct Error {
    error_type: ErrorType,
}

trait ErrorMsg {
    fn get_msg(&self) -> String;
}

#[derive(ThisError, Debug)]
pub enum ErrorType {
    #[error(transparent)]
    Lox(#[from] LoxError),
    #[error(transparent)]
    System(#[from] SystemError),
    #[error(transparent)]
    CLI(#[from] CLIError),
}

#[derive(ThisError, Debug, PartialEq, Eq)]
pub enum LoxError {
    #[error("Unexpected character.")]
    UnexpectedChar(usize),
    #[error("Unterminated string.")]
    UnterminatedString(usize),
    #[error("{1}")]
    CustomError(usize, String),
    #[error("Unknown Type.")]
    UnknownType(usize),
}

impl LoxError {
    fn val(&self) -> usize {
        match self {
            LoxError::UnexpectedChar(val)
            | LoxError::UnknownType(val)
            | LoxError::UnterminatedString(val) => *val,
            | LoxError::CustomError(val, _) => *val
        }
    }
}

impl ErrorMsg for LoxError {
    fn get_msg(&self) -> String {
        format!("LOX | [line {}] {}", self.val(), self.to_string())
    }
}

#[derive(ThisError, Debug)]
pub enum SystemError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("File not found in path: '{0}'")]
    FileNotFound(String),
    #[error("Filed to create file in path: '{0}'")]
    FiledToCreateFile(String),
    #[error("Invalid file extension, expected '.lox' extension")]
    InvalidFileExtension,
}

impl ErrorMsg for SystemError {
    fn get_msg(&self) -> String {
        format!("SYS | {}", self.to_string())
    }
}

#[derive(ThisError, Debug)]
pub enum CLIError {
    #[error("Syntax invalid in AST tool")]
    ASTSyntaxInvalid
}

impl ErrorMsg for CLIError {
    fn get_msg(&self) -> String {
        format!("CLI | {}", self.to_string())
    }
}

impl From<LoxError> for Error {
    fn from(error: LoxError) -> Self {
        Error {
            error_type: ErrorType::Lox(error),
        }
    }
}

impl From<SystemError> for Error {
    fn from(error: SystemError) -> Self {
        Error {
            error_type: ErrorType::System(error),
        }
    }
}

impl From<CLIError> for Error {
    fn from(error: CLIError) -> Self {
        Error {
            error_type: ErrorType::CLI(error),
        }
    }
}
impl Error {
    pub fn report(self) {
        match self.error_type {
            ErrorType::Lox(err) => {
                Alert::error(err.get_msg()).show();
            }
            ErrorType::System(err) => {
                Alert::error(err.get_msg()).show();
            }
            ErrorType::CLI(err) => {
                Alert::error(err.get_msg()).show();
            }
        };
    }

    pub fn report_and_exit(self, code: i32) -> ! {
        match self.error_type {
            ErrorType::Lox(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
            ErrorType::System(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
            ErrorType::CLI(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
        };
    }
}
