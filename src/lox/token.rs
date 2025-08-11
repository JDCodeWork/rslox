use std::fmt;

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: isize,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type
            && self.lexeme == other.lexeme
            && self.line == other.line
    }
}

impl Eq for Token {}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: isize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}

impl Token {
    pub fn get_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn get_literal_as_string(&self) -> Option<String> {
        match &self.token_type {
            TokenType::String(val) => Some(val.clone()),
            TokenType::Number(val) => Some(val.to_string()),
            _ => None,
        }
    }

    pub fn get_lexeme(&self) -> String {
        self.lexeme.clone()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let literal = match self.get_literal_as_string() {
            Some(val) => val,
            None => String::new(),
        };

        write!(
            f,
            "Token( type: {:?}, literal: ({}), lexeme: {} ) at line {}",
            self.token_type, literal, self.lexeme, self.line
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}
