use std::fmt;
use thiserror::Error;

use crate::cli::alerts::Alert;

#[derive(Debug)]
pub struct Located<T> {
    pub error: T,
    pub line: usize,
}

impl<T: fmt::Display> fmt::Display for Located<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] {}", self.line, self.error)
    }
}

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("SCAN | {0}")]
    Scan(Located<ScanError>),
    #[error("PARSE | {0}")]
    Parse(Located<ParseError>),
    #[error("RUNTIME | {0}")]
    Runtime(Located<RuntimeError>),
    #[error("SYS | {0}")]
    Io(#[from] IoError),
}

impl LoxError {
    pub fn report(&self) {
        Alert::error(self.to_string()).show();
    }

    pub fn report_and_exit(&self, code: i32) -> ! {
        self.report();
        std::process::exit(code);
    }
}

pub trait LocateResult<T, E> {
    fn at(self, line: usize) -> Result<T, LoxError>;
}

impl<T> LocateResult<T, ScanError> for Result<T, ScanError> {
    fn at(self, line: usize) -> Result<T, LoxError> {
        self.map_err(|e| LoxError::Scan(Located { error: e, line }))
    }
}

impl<T> LocateResult<T, ParseError> for Result<T, ParseError> {
    fn at(self, line: usize) -> Result<T, LoxError> {
        self.map_err(|e| LoxError::Parse(Located { error: e, line }))
    }
}

impl<T> LocateResult<T, RuntimeError> for Result<T, RuntimeError> {
    fn at(self, line: usize) -> Result<T, LoxError> {
        self.map_err(|e| LoxError::Runtime(Located { error: e, line }))
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ScanError {
    #[error("Unexpected character '{0}'.")]
    UnexpectedChar(char),
    #[error("Unterminated string.")]
    UnterminatedString,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("{0}")]
    ExpectationFailed(String),
    #[error("Unexpected end of input.")]
    UnexpectedEOF,
    #[error("{0} can't have more than 255 arguments.")]
    TooManyArguments(String),
    #[error("Can't read local variable in its own initializer.")]
    SelfReferencingInitializer,
    #[error("Already a variable with this name in this scope.")]
    VariableAlreadyDefined,
    #[error("Can't return from top-level code.")]
    TopLevelReturn,
}

#[derive(Error, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("Operand must be a number.")]
    NumberExpected,
    #[error("Operands must be two numbers or two strings.")]
    InvalidBinaryOperands,
    #[error("Division by zero.")]
    DivisionByZero,
    #[error("Undefined variable \"{0}\".")]
    UndefinedVariable(String),
    #[error("Invalid assignment target.")]
    InvalidAssignment,
    #[error("Can only call functions and classes.")]
    NotCallable,
    #[error("Expected {0} arguments, but got {1}.")]
    ArgumentCountMismatch(usize, usize),
    #[error("Only instances have properties.")]
    NotAnInstance,
}

#[derive(Error, Debug)]
pub enum IoError {
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

pub trait Locate {
    fn at(self, line: usize) -> LoxError;
}

impl Locate for ScanError {
    fn at(self, line: usize) -> LoxError {
        LoxError::Scan(Located { error: self, line })
    }
}

impl Locate for ParseError {
    fn at(self, line: usize) -> LoxError {
        LoxError::Parse(Located { error: self, line })
    }
}

impl Locate for RuntimeError {
    fn at(self, line: usize) -> LoxError {
        LoxError::Runtime(Located { error: self, line })
    }
}
