use thiserror::Error as ThisError;

use crate::{cli::alerts::Alert, lox::token::Token};

mod working_on {
    pub enum Err {}


}

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
}

trait ErrorMsg {
    fn get_msg(&self) -> String;
}

#[derive(ThisError, Debug)]
pub enum ErrorType {
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    System(#[from] SystemError),
    #[error(transparent)]
    CLI(#[from] CLIError),
}

#[derive(ThisError, Debug, PartialEq, Eq)]
pub enum ScanError {
    #[error("Unexpected character.")]
    UnexpectedChar(usize),
    #[error("Unterminated string.")]
    UnterminatedString(usize),
}

impl ScanError {
    fn line(&self) -> usize {
        match self {
            ScanError::UnexpectedChar(line) | ScanError::UnterminatedString(line) => *line,
        }
    }
}

impl ErrorMsg for ScanError {
    fn get_msg(&self) -> String {
        format!("SCAN | [line {}] {}", self.line(), self.to_string())
    }
}

#[derive(ThisError, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("Empty input.")]
    EmptyInput,
    #[error("Unknown token type.")]
    UnknownTokenType(usize),
    #[error("Expect '{0}'.")]
    ExpectedToken(String, usize),
}

impl ParseError {
    fn position(&self) -> Option<usize> {
        match self {
            ParseError::EmptyInput => None,
            ParseError::UnknownTokenType(pos) | ParseError::ExpectedToken(_, pos) => Some(*pos),
        }
    }
}

impl ErrorMsg for ParseError {
    fn get_msg(&self) -> String {
        if let Some(pos) = self.position() {
            format!("PARSE | [token {}] {}", pos, self.to_string())
        } else {
            format!("PARSE | {}", self.to_string())
        }
    }
}

// ===== Runtime Errors =====
#[derive(ThisError, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("Operand must be a number.")]
    OperandMustBeNumber,
    #[allow(dead_code)]
    #[error("Operands must be two numbers or two strings.")]
    InvalidOperandTypes,
    #[error("Division by zero.")]
    DivisionByZero,
    #[error("Cannot apply operator '{0}' to the given operands.")]
    InvalidOperation(String),
}

impl ErrorMsg for RuntimeError {
    fn get_msg(&self) -> String {
        format!("RUNTIME | {}", self.to_string())
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
    ASTSyntaxInvalid,
}

impl ErrorMsg for CLIError {
    fn get_msg(&self) -> String {
        format!("CLI | {}", self.to_string())
    }
}

// ===== From implementations =====
impl From<ScanError> for Error {
    fn from(error: ScanError) -> Self {
        Error {
            error_type: ErrorType::Scan(error),
        }
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error {
            error_type: ErrorType::Parse(error),
        }
    }
}

impl From<RuntimeError> for Error {
    fn from(error: RuntimeError) -> Self {
        Error {
            error_type: ErrorType::Runtime(error),
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
            ErrorType::Scan(err) => {
                Alert::error(err.get_msg()).show();
            }
            ErrorType::Parse(err) => {
                Alert::error(err.get_msg()).show();
            }
            ErrorType::Runtime(err) => {
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
            ErrorType::Scan(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
            ErrorType::Parse(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
            ErrorType::Runtime(err) => {
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
