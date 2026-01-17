use std::collections::HashMap;

use crate::errors::{Locate, ScanError};

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

    pub fn scan_from(source: String) -> Vec<Token> {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().clone()
    }
}

impl Scanner {
    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EOF, String::new(), self.line));
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
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    loop {
                        if self.peek() == '*' && self.peek_next() == '/' {
                            break;
                        }

                        // Handle EOF inside block comment
                        if self.is_at_end() {
                            ScanError::UnterminatedString.at(self.line).report(); // Using UnterminatedString error for block comment? 
                            return;
                        }

                        if self.peek() == '\n' {
                            self.line += 1;
                        }

                        self.advance();
                    }

                    // The closing */
                    for _ in 0..2 {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {} // Ignore whitespace
            '\n' => self.line += 1,
            '"' => self.string(),
            ch if ch.is_ascii_digit() => self.number(),
            ch if ch.is_ascii_alphabetic() || ch == '_' => self.identifier(),
            ch => {
                ScanError::UnexpectedChar(ch).at(self.line).report();
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
        tokens.push(Token::new(token_type, text.to_string(), *line));
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
            ScanError::UnterminatedString.at(self.line).report();
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
            Err(..) => 0.0, // Fallback value, shouldn't happen with valid scan
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_character_tokens() {
        let mut scanner = Scanner::new("(){},.+-;*".to_string());
        let tokens = scanner.scan_tokens();

        let expected_types = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Semicolon,
            TokenType::Star,
            TokenType::EOF,
        ];

        assert_eq!(tokens.len(), expected_types.len());

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(&tokens[i].type_, expected_type);
        }
    }

    #[test]
    fn test_two_character_tokens() {
        let mut scanner = Scanner::new("!= == <= >=".to_string());
        let tokens = scanner.scan_tokens();

        let expected_types = vec![
            TokenType::BangEqual,
            TokenType::EqualEqual,
            TokenType::LessEqual,
            TokenType::GreaterEqual,
            TokenType::EOF,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(&tokens[i].type_, expected_type);
        }
    }

    #[test]
    fn test_string_literals() {
        let mut scanner = Scanner::new(r#""hello" "world with spaces""#.to_string());
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens.len(), 3); // two strings + EOF

        match &tokens[0].type_ {
            TokenType::String(value) => assert_eq!(value, "hello"),
            _ => panic!("Expected string token"),
        }

        match &tokens[1].type_ {
            TokenType::String(value) => assert_eq!(value, "world with spaces"),
            _ => panic!("Expected string token"),
        }
    }

    #[test]
    fn test_number_literals() {
        let mut scanner = Scanner::new("123 456.789 0.5".to_string());
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens.len(), 4); // three numbers + EOF

        match tokens[0].type_ {
            TokenType::Number(value) => assert_eq!(value, 123.0),
            _ => panic!("Expected number token"),
        }

        match tokens[1].type_ {
            TokenType::Number(value) => assert_eq!(value, 456.789),
            _ => panic!("Expected number token"),
        }

        match tokens[2].type_ {
            TokenType::Number(value) => assert_eq!(value, 0.5),
            _ => panic!("Expected number token"),
        }
    }

    #[test]
    fn test_keywords() {
        let mut scanner = Scanner::new("if else true false nil".to_string());
        let tokens = scanner.scan_tokens();

        let expected_types = vec![
            TokenType::If,
            TokenType::Else,
            TokenType::True,
            TokenType::False,
            TokenType::Nil,
            TokenType::EOF,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(&tokens[i].type_, expected_type);
        }
    }

    #[test]
    fn test_identifiers() {
        let mut scanner = Scanner::new("variable_name function123 _underscore".to_string());
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens.len(), 4); // three identifiers + EOF

        for i in 0..3 {
            assert_eq!(&tokens[i].type_, &TokenType::Identifier);
        }

        assert_eq!(tokens[0].lexeme, "variable_name");
        assert_eq!(tokens[1].lexeme, "function123");
        assert_eq!(tokens[2].lexeme, "_underscore");
    }

    #[test]
    fn test_comments() {
        let mut scanner = Scanner::new("// this is a comment\n123".to_string());
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens.len(), 2); // number + EOF (comment ignored)

        match tokens[0].type_ {
            TokenType::Number(value) => assert_eq!(value, 123.0),
            _ => panic!("Expected number token"),
        }
    }

    #[test]
    fn test_block_comments() {
        let mut scanner = Scanner::new("/* block comment */ 456".to_string());
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens.len(), 2); // number + EOF (comment ignored)

        match tokens[0].type_ {
            TokenType::Number(value) => assert_eq!(value, 456.0),
            _ => panic!("Expected number token"),
        }
    }

    #[test]
    fn test_line_counting() {
        let mut scanner = Scanner::new("123\n456\n789".to_string());
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[2].line, 3);
    }

    #[test]
    fn test_expression() {
        let mut scanner = Scanner::new("1 + 2 * (3 - 4)".to_string());
        let tokens = scanner.scan_tokens();

        let expected_types = vec![
            TokenType::Number(1.0),
            TokenType::Plus,
            TokenType::Number(2.0),
            TokenType::Star,
            TokenType::LeftParen,
            TokenType::Number(3.0),
            TokenType::Minus,
            TokenType::Number(4.0),
            TokenType::RightParen,
            TokenType::EOF,
        ];

        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(&tokens[i].type_, expected_type);
        }
    }
}
