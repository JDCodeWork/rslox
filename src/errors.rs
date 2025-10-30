use thiserror::Error as ThisError;

use crate::cli::alerts::Alert;

trait ErrorMsg {
    fn get_msg(&self) -> String;
}

#[derive(ThisError, Debug)]
pub enum Err {
    #[error(transparent)]
    Scan(#[from] ScanErr),
    #[error(transparent)]
    Parse(#[from] ParseErr),
    #[error(transparent)]
    Runtime(#[from] RuntimeErr),
    #[error(transparent)]
    Io(#[from] IoErr),
}

#[derive(ThisError, Debug, PartialEq)]
pub enum ScanErr {
    #[error("Unexpected character '{0}'.")]
    UnexpectedChar(char, usize),
    #[error("Unterminated string.")]
    UnterminatedString(usize),
}

impl ScanErr {
    fn ln(&self) -> usize {
        match self {
            ScanErr::UnexpectedChar(_, line) | ScanErr::UnterminatedString(line) => *line,
        }
    }

    pub fn to_err(self) -> Err {
        Err::Scan(self)
    }
}

impl ErrorMsg for ScanErr {
    fn get_msg(&self) -> String {
        format!("SCAN | [line {}] {}", self.ln(), self.to_string())
    }
}

#[derive(ThisError, Debug, PartialEq)]
pub enum ParseErr {
    #[error("Expect '{0}'.")]
    ExpectedToken(String, usize),
    #[error("Unexpected end of input.")]
    UnexpectedEOF(usize),
}

impl ParseErr {
    fn ln(&self) -> Option<usize> {
        match self {
            ParseErr::ExpectedToken(_, ln) => Some(*ln),
            ParseErr::UnexpectedEOF(ln) => Some(*ln),
        }
    }

    pub fn to_err(self) -> Err {
        Err::Parse(self)
    }
}

impl ErrorMsg for ParseErr {
    fn get_msg(&self) -> String {
        if let Some(ln) = self.ln() {
            format!("PARSE | [line {}] {}", ln, self.to_string())
        } else {
            format!("PARSE | {}", self.to_string())
        }
    }
}

// ===== Runtime Errors =====
#[derive(ThisError, Debug, PartialEq)]
pub enum RuntimeErr {
    #[error("Operand must be a number.")]
    OperandMustBeNumber,
    #[error("Operands must be two numbers or two strings.")]
    InvalidOperandTypes,
    #[error("Division by zero.")]
    DivisionByZero,
}

impl RuntimeErr {
    pub fn to_err(self) -> Err {
        Err::Runtime(self)
    }
}

impl ErrorMsg for RuntimeErr {
    fn get_msg(&self) -> String {
        format!("RUNTIME | {}", self.to_string())
    }
}

#[derive(ThisError, Debug)]
pub enum IoErr {
    #[error(transparent)]
    Sys(#[from] std::io::Error),
    #[error("File not found in path: '{0}'")]
    FileNotFound(String),
    #[error("Failed to create file in path: '{0}'")]
    FailedToCreateFile(String),
    #[error("Invalid file extension, expected '.lox' extension")]
    InvalidFileExtension,
    #[error("Syntax invalid in AST tool")]
    ASTSyntaxInvalid,
}

impl ErrorMsg for IoErr {
    fn get_msg(&self) -> String {
        format!("SYS | {}", self.to_string())
    }
}

impl IoErr {
    pub fn to_err(self) -> Err {
        Err::Io(self)
    }
}

// ===== From implementations =====
impl Err {
    pub fn report(self) {
        match self {
            Err::Scan(err) => {
                Alert::error(err.get_msg()).show();
            }
            Err::Parse(err) => {
                Alert::error(err.get_msg()).show();
            }
            Err::Runtime(err) => {
                Alert::error(err.get_msg()).show();
            }
            Err::Io(err) => {
                Alert::error(err.get_msg()).show();
            }
        };
    }

    pub fn report_and_exit(self, code: i32) -> ! {
        match self {
            Err::Scan(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
            Err::Parse(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
            Err::Runtime(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
            Err::Io(err) => {
                Alert::error(err.get_msg()).show_and_exit(code);
            }
        };
    }
}
