use std::{any, fmt};

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Box<dyn any::Any>>,
    line: isize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Box<dyn any::Any>>,
        line: isize,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Token {
    pub fn get_type(&self) -> &TokenType {
        &self.token_type
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let literal = if let Some(val) = &self.literal {
            if let Some(lit) = val.downcast_ref::<String>() {
                lit.clone()
            } else if let Some(lit) = val.downcast_ref::<f64>() {
                lit.to_string()
            } else {
                String::from("Unknown type")
            }
        } else {
            String::new()
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
    String,
    Number,

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
