use std::any::Any;

use crate::{
    errors::{Error, LoxError},
    token::{Token, TokenType},
};

#[derive(Debug)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
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
            None,
            self.line as isize,
        ));
        &self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                let token_type = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };

                self.add_token(token_type, None)
            }
            '=' => {
                let token_type = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };

                self.add_token(token_type, None)
            }
            '<' => {
                let token_type = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };

                self.add_token(token_type, None)
            }
            '>' => {
                let token_type = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.add_token(token_type, None)
            }
            '/' => {
                if self.match_char('/') {
                    while self.pick() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            _ => {
                Error::from(LoxError::UnexpectedChar)
                    .with_line(self.line as isize)
                    .report();
            }
        };
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Box<dyn Any>>) {
        let Self {
            current,
            start,
            source,
            tokens,
            line,
        } = self;

        let text = &source[*start..*current];
        tokens.push(Token::new(
            token_type,
            text.to_string(),
            literal,
            *line as isize,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        match self.source.chars().nth(self.current) {
            Some(c) => {
                self.current += 1;
                c
            }
            None => ' ',
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let found = match self.source.chars().nth(self.current) {
            Some(c) => c == expected,
            None => false,
        };

        if found {
            self.current += 1;
        }

        found
    }

    fn pick(&self) -> char {
        match self.source.chars().nth(self.current) {
            Some(c) => c,
            None => '\0',
        }
    }
}
