use crate::{chunk::MarshalError, scanner::Scanner};
use std::{collections::HashMap, str::FromStr};

use crate::{
    chunk::{Byte, Chunk, OpCode},
    scanner::{Token, TokenKind},
};

#[derive(Debug, Clone, Copy)]
enum PrefixRule {
    Unary,
    Grouping,
    Number,
}

#[derive(Debug, Clone, Copy)]
enum InfixRule {
    Binary,
}

struct ParseRule {
    prefix: Option<PrefixRule>,
    infix: Option<InfixRule>,
    precedence: Precedence,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assign,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

struct Parser {
    prev: Token,
    curr: Token,

    rules: HashMap<TokenKind, ParseRule>,

    had_err: bool,
    panic_mode: bool,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            prev: Token::default(),
            curr: Token::default(),
            rules: HashMap::new(),
            had_err: false,
            panic_mode: false,
        }
    }
}

pub struct Compiler<'a> {
    parser: Parser,
    scanner: Scanner<'a>,

    chunk: &'a mut Chunk,
    source: &'a str,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str, chunk: &'a mut Chunk) -> Self {
        let mut parser = Parser::default();
        parser.define_rules();

        let scanner = Scanner::new(source);

        Self {
            parser,
            scanner,
            source,
            chunk,
        }
    }

    pub fn compile(&mut self) -> bool {
        self.advance();
        self.expression();

        self.consume(TokenKind::EOF, "Expect end of expression");
        self.emit_byte(OpCode::Return);

        !self.parser.had_err
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assign);
    }

    fn number(&mut self) {
        let str = self.parser.prev.lexeme(self.source);
        // The scanner has the job of ensuring that the lexeme is a number
        let val = f64::from_str(str).unwrap();
        let const_ = self.make_constant(val);

        self.emit_bytes(OpCode::Constant as u8, const_);
    }

    fn unary(&mut self) {
        let op = self.parser.prev.kind;
        self.parse_precedence(Precedence::Unary);

        match op {
            TokenKind::Minus => self.emit_byte(OpCode::Negate),
            _ => {}
        };
    }

    fn binary(&mut self) {
        let op = self.parser.prev.kind;
        let precedence = self.parser_rule_from(&op).precedence;
        self.parse_precedence(
            Precedence::try_from(precedence as u8 + 1).unwrap_or(Precedence::None),
        );

        match op {
            TokenKind::Plus => self.emit_byte(OpCode::Add),
            TokenKind::Minus => self.emit_byte(OpCode::Sub),
            TokenKind::Star => self.emit_byte(OpCode::Mul),
            TokenKind::Slash => self.emit_byte(OpCode::Div),
            _ => {}
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();
        let prefix_option = self.parser_rule_from(&self.parser.prev.kind).prefix;
        let Some(prefix) = prefix_option else {
            self.error_at(self.parser.curr, "Expect expression.");
            return;
        };

        self.compile_prefix(prefix);
        while prec <= self.parser_rule_from(&self.parser.curr.kind).precedence {
            self.advance();
            if let Some(infix) = self.parser_rule_from(&self.parser.prev.kind).infix {
                self.compile_infix(infix);
            };
        }
    }

    fn make_constant(&mut self, val: f64) -> Byte {
        let const_idx = self.chunk.add_const(val);

        if const_idx > u8::MAX as usize {
            self.error_at(self.parser.curr, "Too many constants in a chunk");

            0
        } else {
            const_idx as u8
        }
    }

    fn compile_prefix(&mut self, prefix: PrefixRule) {
        match prefix {
            PrefixRule::Grouping => self.grouping(),
            PrefixRule::Number => self.number(),
            PrefixRule::Unary => self.unary(),
        }
    }

    fn compile_infix(&mut self, infix: InfixRule) {
        match infix {
            InfixRule::Binary => self.binary(),
        }
    }

    fn parser_rule_from(&self, op: &TokenKind) -> &ParseRule {
        if let Some(prule) = self.parser.rules.get(op) {
            prule
        } else {
            // Safety while the rules in parser are initiaized
            self.parser.rules.get(&TokenKind::EOF).unwrap()
        }
    }

    fn advance(&mut self) {
        self.parser.prev = self.parser.curr;

        loop {
            let instr = self.scanner.scan_token();

            if let Err(sc_err) = instr {
                self.error(format!("{sc_err}").as_str());
            } else {
                self.parser.curr = instr.unwrap();
                break;
            }
        }
    }

    fn consume(&mut self, kind: TokenKind, error: &str) {
        if self.parser.curr.kind == kind {
            self.advance();
            return;
        }
        self.error_at(self.parser.curr, error);
    }

    fn emit_byte<B: Into<u8>>(&mut self, b: B) {
        self.chunk.write(b.into(), self.parser.prev.line);
    }

    fn emit_bytes<B: Into<u8>>(&mut self, b1: B, b2: B) {
        self.emit_byte(b1);
        self.emit_byte(b2);
    }

    fn error_at(&mut self, at: Token, msg: &str) {
        let str = format!(
            "Error: {} \n--> {}:{} to {}:{}",
            msg, at.line, at.span.start, at.line, at.span.end
        );

        self.error(str.as_str());
    }

    fn error(&mut self, msg: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;

        eprintln!("{msg}");

        self.parser.had_err = true;
    }
}

impl Default for ParseRule {
    fn default() -> Self {
        Self {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }
    }
}

#[allow(dead_code)]
impl ParseRule {
    fn prefix(mut self, p: PrefixRule) -> Self {
        self.prefix = Some(p);
        self
    }

    fn infix(mut self, i: InfixRule) -> Self {
        self.infix = Some(i);
        self
    }

    fn precedence(mut self, p: Precedence) -> Self {
        self.precedence = p;
        self
    }
}

impl TryFrom<u8> for Precedence {
    type Error = MarshalError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > (Precedence::Primary as u8) - 1 {
            return Err(MarshalError::InvalidPrecedence);
        }

        let opcode = unsafe { core::mem::transmute::<Byte, Self>(value) };
        Ok(opcode)
    }
}

impl Parser {
    fn define_rules(&mut self) {
        self.rules.insert(
            TokenKind::LeftParen,
            ParseRule::default().prefix(PrefixRule::Grouping),
        );

        self.rules.insert(
            TokenKind::Minus,
            ParseRule::default()
                .prefix(PrefixRule::Unary)
                .infix(InfixRule::Binary)
                .precedence(Precedence::Term),
        );

        self.rules.insert(
            TokenKind::Plus,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Term),
        );

        self.rules.insert(
            TokenKind::Slash,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Factor),
        );
        self.rules.insert(
            TokenKind::Star,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Factor),
        );

        self.rules.insert(
            TokenKind::Number,
            ParseRule::default().prefix(PrefixRule::Number),
        );

        self.rules.insert(TokenKind::EOF, ParseRule::default());
    }
}
