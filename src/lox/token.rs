use std::{any, fmt, rc::Rc};

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Rc<dyn any::Any>>,
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
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Rc<dyn any::Any>>,
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

    pub fn get_literal(&self) -> String {
        match &self.literal {
            None => String::new(),
            Some(literal) => {
                if let Some(string) = literal.downcast_ref::<String>() {
                    string.clone()
                } else if let Some(int) = literal.downcast_ref::<i32>() {
                    int.to_string()
                } else if let Some(float) = literal.downcast_ref::<f64>() {
                    float.to_string()
                } else {
                    String::new()
                }
            }
        }
    }

    pub fn get_lexeme(&self) -> String {
        self.lexeme.clone()
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
