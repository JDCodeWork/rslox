use std::collections::HashMap;

use crate::errors::{Error, LoxError};

use super::token::{Token, TokenType};

#[derive(Debug)]
pub(super) struct Scanner {
    source: String,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

fn keywords() -> HashMap<&'static str, TokenType> {
    let mut keywords = HashMap::new();

    keywords.insert("and", TokenType::And);
    keywords.insert("class", TokenType::Class);
    keywords.insert("else", TokenType::Else);
    keywords.insert("false", TokenType::False);
    keywords.insert("for", TokenType::For);
    keywords.insert("fun", TokenType::Fun);
    keywords.insert("if", TokenType::If);
    keywords.insert("nil", TokenType::Nil);
    keywords.insert("or", TokenType::Or);
    keywords.insert("print", TokenType::Print);
    keywords.insert("return", TokenType::Return);
    keywords.insert("super", TokenType::Super);
    keywords.insert("this", TokenType::This);
    keywords.insert("true", TokenType::True);
    keywords.insert("var", TokenType::Var);
    keywords.insert("while", TokenType::While);

    keywords
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
}

impl Scanner {
    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            String::new(),
            self.line as isize,
        ));
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token_type = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };

                self.add_token(token_type)
            }
            '=' => {
                let token_type = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };

                self.add_token(token_type)
            }
            '<' => {
                let token_type = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };

                self.add_token(token_type)
            }
            '>' => {
                let token_type = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.add_token(token_type)
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    loop {
                        if self.peek() == '*' && self.peek_next() == '/' {
                            break;
                        }

                        if self.peek() == '\n' {
                            self.line += 1;
                        }

                        self.advance();
                    }

                    if self.is_at_end() {
                        Error::from(LoxError::UnterminatedString(self.line)).report();
                        return;
                    }

                    // The closing */
                    for _ in 0..2 {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string(),
            ch if ch.is_ascii_digit() => self.number(),
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.identifier(),
            _ => {
                Error::from(LoxError::UnexpectedChar(self.line)).report();
            }
        };
    }

    fn add_token(&mut self, token_type: TokenType) {
        let Self {
            current,
            start,
            source,
            tokens,
            line,
        } = self;

        let text = &source[*start..*current];
        tokens.push(Token::new(token_type, text.to_string(), *line as isize));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let current_char = match self.source[self.current..].chars().next() {
            Some(c) => c,
            None => '\0',
        };

        self.current += current_char.len_utf8();

        current_char
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let found = match self.source[self.current..].chars().nth(0) {
            Some(c) => c == expected,
            None => false,
        };

        if found {
            self.current += 1;
        }

        found
    }

    fn peek(&self) -> char {
        match self.source[self.current..].chars().next() {
            Some(c) => c,
            None => '\0',
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            Error::from(LoxError::UnterminatedString(self.line)).report();
            return;
        }

        // The closing "
        self.advance();

        // Trim the surrounding quotes
        let literal = &self.source[(self.start + 1)..(self.current - 1)];
        self.add_token(TokenType::String(literal.to_string()));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let literal: f64 = match &self.source[self.start..self.current].parse() {
            Ok(n) => *n,
            Err(..) => {
                Error::from(LoxError::UnknownType(self.line)).report_and_exit(1);
            }
        };
        self.add_token(TokenType::Number(literal));
    }

    fn peek_next(&self) -> char {
        match self.source[self.current..].chars().nth(1) {
            Some(c) => c,
            None => '\0',
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        let token_type: TokenType = if let Some(token) = keywords().get(text) {
            token.clone()
        } else {
            TokenType::Identifier
        };

        self.add_token(token_type);
    }
}
