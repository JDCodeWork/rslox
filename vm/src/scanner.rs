pub struct Scanner<'a> {
    pub src: &'a [u8],
    pub start: usize,
    pub curr: usize,
    pub line: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub span: Span,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            kind: TokenKind::Nil,
            line: 0,
            span: Span { start: 0, end: 0 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TokenKind {
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

impl<'a> Scanner<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            start: 0,
            curr: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, ScannerError> {
        self.skip_whitespace();

        self.start = self.curr;

        let Some(c) = self.advance() else {
            return Ok(self.make_token(TokenKind::EOF));
        };

        if (*c as char).is_numeric() {
            return Ok(self.number());
        }

        if (*c as char).is_alphabetic() {
            return Ok(self.identifier());
        }

        let t = match *c as char {
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            ';' => self.make_token(TokenKind::Semicolon),
            ',' => self.make_token(TokenKind::Comma),
            '.' => self.make_token(TokenKind::Dot),
            '-' => self.make_token(TokenKind::Minus),
            '+' => self.make_token(TokenKind::Plus),
            '/' => self.make_token(TokenKind::Slash),
            '*' => self.make_token(TokenKind::Star),
            '!' => {
                if self.match_('=') {
                    self.make_token(TokenKind::BangEqual)
                } else {
                    self.make_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.match_('=') {
                    self.make_token(TokenKind::EqualEqual)
                } else {
                    self.make_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.match_('=') {
                    self.make_token(TokenKind::LessEqual)
                } else {
                    self.make_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.match_('=') {
                    self.make_token(TokenKind::GreaterEqual)
                } else {
                    self.make_token(TokenKind::Greater)
                }
            }
            '"' => return self.string(),
            _ => return Err(self.make_err("Unexpected character.")),
        };

        Ok(t)
    }

    fn number(&mut self) -> Token {
        while byte_to_char_or(self.peek(), ' ').is_numeric() {
            self.advance();
        }

        if let Some(b'.') = self.peek() {
            while byte_to_char_or(self.peek(), ' ').is_numeric() {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn string(&mut self) -> Result<Token, ScannerError> {
        while byte_to_char_or(self.peek(), '"') != '"' {
            self.advance();
        }

        if let None = self.peek() {
            return Err(self.make_err("Unterminated string."));
        }

        self.advance(); // consume "

        Ok(self.make_token(TokenKind::String))
    }

    fn identifier(&mut self) -> Token {
        while byte_to_char_or(self.peek(), ' ').is_alphanumeric() {
            self.advance();
        }

        let kind = self.identifier_kind();
        self.make_token(kind)
    }

    fn identifier_kind(&mut self) -> TokenKind {
        match self.src[self.start] as char {
            'a' => return self.check_keyword(1, 2, "nd", TokenKind::And),
            'c' => return self.check_keyword(1, 4, "lass", TokenKind::Class),
            'e' => return self.check_keyword(1, 3, "lse", TokenKind::Else),
            'i' => return self.check_keyword(1, 1, "f", TokenKind::If),
            'n' => return self.check_keyword(1, 2, "il", TokenKind::Nil),
            'o' => return self.check_keyword(1, 1, "r", TokenKind::Or),
            'p' => return self.check_keyword(1, 4, "rint", TokenKind::Print),
            'r' => return self.check_keyword(1, 5, "eturn", TokenKind::Return),
            's' => return self.check_keyword(1, 4, "uper", TokenKind::Super),
            'v' => return self.check_keyword(1, 2, "ar", TokenKind::Var),
            'w' => return self.check_keyword(1, 4, "hile", TokenKind::While),
            'f' => {
                if self.curr - self.start > 1 {
                    match self.src[self.start + 1] as char {
                        'a' => return self.check_keyword(2, 3, "lse", TokenKind::False),
                        'o' => return self.check_keyword(2, 1, "r", TokenKind::For),
                        'u' => return self.check_keyword(2, 1, "n", TokenKind::Fun),
                        _ => {}
                    }
                }
            }
            't' => {
                if self.curr - self.start > 1 {
                    match self.src[self.start + 1] as char {
                        'h' => return self.check_keyword(2, 2, "is", TokenKind::This),
                        'r' => return self.check_keyword(2, 2, "ue", TokenKind::True),
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        TokenKind::Identifier
    }

    fn check_keyword(
        &mut self,
        start: usize,
        length: usize,
        rest: &str,
        kind: TokenKind,
    ) -> TokenKind {
        if
        // If both lexeme and expected have the same length
        self.curr - self.start == start + length
            // If both lexeme bytes and expected bytes are the same
            && self.src[self.start + start..self.start + start + length] == *rest.as_bytes()
        {
            return kind;
        }

        TokenKind::Identifier
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            line: self.line,
            span: Span {
                start: self.start,
                end: self.curr,
            },
        }
    }

    fn advance(&mut self) -> Option<&u8> {
        self.curr += 1;

        self.src.get(self.curr - 1)
    }

    fn peek(&self) -> Option<&u8> {
        self.src.get(self.curr)
    }

    fn peek_next(&self) -> Option<&u8> {
        self.src.get(self.curr + 1)
    }

    fn match_(&mut self, expect: char) -> bool {
        let Some(c) = self.src.get(self.curr) else {
            return false;
        };

        *c as char == expect
    }

    fn skip_whitespace(&mut self) {
        loop {
            let Some(c) = self.peek() else {
                break;
            };

            match *c as char {
                ' ' | '\t' | '\r' => self.advance(),
                '/' => {
                    if let Some(b'/') = self.peek_next() {
                        while *self.peek().unwrap_or(&b'\n') != b'\n' {
                            self.advance();
                        }
                    }
                    break;
                }
                '\n' => {
                    self.line += 1;
                    self.advance()
                }
                _ => break,
            };
        }
    }

    fn make_err(&self, desc: &str) -> ScannerError {
        ScannerError {
            desc: desc.to_string(),
            line: self.line,
            start: self.start,
            end: self.curr,
        }
    }
}

impl Token {
    pub fn lexeme<'a>(&self, src: &'a str) -> &'a str {
        if self.span.end > src.len() {
            return " "; // Lexeme for EOF
        }

        &src[self.span.start..self.span.end]
    }
}

#[inline]
/// Converts an optional byte to a char, using a default char if None
fn byte_to_char_or(byte: Option<&u8>, default_char: char) -> char {
    (*byte.unwrap_or(&(default_char as u8))) as char
}

#[derive(Debug)]
pub struct ScannerError {
    pub desc: String,
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl core::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "Error: {} \n--> {}:{} to {}:{}",
                self.desc, self.line, self.start, self.line, self.end
            )
            .as_str(),
        )
    }
}

impl core::error::Error for ScannerError {}
