use crate::{scanner::Scanner, values::Constant};
use std::{collections::HashMap, str::FromStr, u8};

use crate::{
    chunk::{Byte, Chunk, OpCode},
    scanner::{Token, TokenKind},
};

#[derive(Debug, Clone, Copy)]
enum PrefixRule {
    Unary,
    Grouping,
    Number,
    Literal,
    String,
    Variable,
}

#[derive(Debug, Clone, Copy)]
enum InfixRule {
    Binary,
    And,
    Or,
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
    _Call,
    Primary,
}

struct Parser {
    prev: Token,
    curr: Token,

    rules: HashMap<TokenKind, ParseRule>,
    can_assign: bool,

    had_err: bool,
    panic_mode: bool,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            prev: Token::default(),
            curr: Token::default(),
            rules: HashMap::new(),
            can_assign: false,
            had_err: false,
            panic_mode: false,
        }
    }
}

struct Local {
    name: Token,
    depth: usize,
    ready: bool,
}

#[derive(Default)]
struct CompilerContext {
    locals: Vec<Local>,
    scope: usize,
}

impl CompilerContext {
    const MAX_LOCALS: usize = (u8::MAX as usize) + 1;

    fn add_local(&mut self, name: Token) -> Result<(), &'static str> {
        if self.locals.len() == Self::MAX_LOCALS {
            return Err("Too many local variables in function.");
        }

        let local = Local {
            name,
            depth: self.scope,
            ready: false,
        };

        self.locals.push(local);

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scope += 1;
    }

    fn end_scope(&mut self) -> usize {
        self.scope -= 1;

        let mut pops = 0;
        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope {
                break;
            }

            self.locals.pop();
            pops += 1;
        }

        pops
    }
}

pub struct Compiler<'a> {
    parser: Parser,
    scanner: Scanner<'a>,

    chunk: &'a mut Chunk,
    context: CompilerContext,
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
            context: CompilerContext::default(),
        }
    }

    pub fn compile(&mut self) -> bool {
        self.advance();

        while !self._match(TokenKind::EOF) {
            self.declaration();
        }
        self.emit_byte(OpCode::Return);

        !self.parser.had_err
    }

    fn declaration(&mut self) {
        if self._match(TokenKind::Var) {
            self.var_decl();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.syncronize();
        }
    }

    fn var_decl(&mut self) {
        let var = self.parse_var();

        if self._match(TokenKind::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaratiion.",
        );
        self.def_var(var);
    }

    fn def_var(&mut self, var: Byte) {
        if self.context.scope > 0 {
            if let Some(local) = self.context.locals.last_mut() {
                local.ready = true;
            }

            return;
        }

        self.emit_bytes(OpCode::DefGlob as u8, var);
    }

    fn parse_var(&mut self) -> Byte {
        self.consume(
            TokenKind::Identifier,
            "Expect variable name after 'var' keyword.",
        );

        let name = self.parser.prev.span;
        self.declare_var();
        if self.context.scope > 0 {
            return 0;
        }

        self.make_constant(Constant::String {
            start: name.start,
            end: name.end,
        })
    }

    fn declare_var(&mut self) {
        if self.context.scope == 0 {
            return;
        }

        let name = self.parser.prev;
        let mut had_err = None;

        for local in self.context.locals.iter().rev() {
            if local.ready && local.depth < self.context.scope {
                break;
            }

            if self.idents_equals(name, local.name) {
                had_err = Some("Already variable with this name in this scope.");
                break;
            }
        }

        if let Some(err) = had_err {
            self.error_at(name, err);
        }

        if let Err(msg) = self.context.add_local(name) {
            self.error_at(name, msg);
        }
    }

    fn variable(&mut self) {
        self.named_variable(self.parser.prev);
    }

    fn named_variable(&mut self, var: Token) {
        let (arg, set_op, get_op) = if let Some(idx) = self.resolve_local(var) {
            (idx, OpCode::SetLocal, OpCode::GetLocal)
        } else {
            let glob = self.make_constant(Constant::String {
                start: var.span.start,
                end: var.span.end,
            });

            (glob, OpCode::SetGlob, OpCode::GetGlob)
        };

        if self.parser.can_assign && self._match(TokenKind::Equal) {
            self.expression();
            self.emit_bytes(set_op as u8, arg);
        } else {
            self.emit_bytes(get_op as u8, arg);
        }
    }

    fn resolve_local(&mut self, name: Token) -> Option<u8> {
        for (idx, local) in self.context.locals.iter().enumerate().rev() {
            if !self.idents_equals(name, local.name) {
                continue;
            }

            if !local.ready {
                self.error_at(name, "Can't read local variable in its own initializer.");
                return None;
            }

            return Some(idx as u8);
        }

        None
    }

    fn statement(&mut self) {
        match self.parser.curr.kind {
            TokenKind::If => self.if_stmt(),
            TokenKind::Print => self.print_stmt(),
            TokenKind::LeftBrace => self.block_stmt(),
            TokenKind::While => self.while_stmt(),
            TokenKind::For => self.for_stmt(),
            _ => self.expression_stmt(),
        }
    }

    fn for_stmt(&mut self) {
        self.advance(); // Consume 'for'
        self.context.begin_scope();

        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.");
        // INITIALIZER
        match self.parser.curr.kind {
            TokenKind::Semicolon => self.advance(),
            TokenKind::Var => {
                self.advance();
                self.var_decl()
            }
            _ => self.expression_stmt(),
        }

        // CONDITION
        let mut for_start = self.chunk.code.len();
        let jump_for = (!self._match(TokenKind::Semicolon)).then(|| {
            self.expression();
            self.consume(TokenKind::Semicolon, "Expect ';' after condition.");

            let offset = self.emit_jump(OpCode::JumpIfFalse);
            offset
        });

        if !self._match(TokenKind::RightParen) {
            let jump_incr = self.emit_jump(OpCode::Jump);
            let incr_start = self.chunk.code.len();

            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(TokenKind::RightParen, "Expect ')' after increment.");

            self.emit_loop(for_start);
            for_start = incr_start;
            self.path_jump(jump_incr);
        }

        self.statement();

        self.emit_loop(for_start);
        if let Some(offset) = jump_for {
            self.path_jump(offset);
            self.emit_byte(OpCode::Pop);
        }

        self.context.end_scope();
    }

    fn while_stmt(&mut self) {
        self.advance(); // Consume 'while'

        let while_start = self.chunk.code.len();

        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let jump_while = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);

        self.statement();
        self.emit_loop(while_start);

        self.path_jump(jump_while);
        self.emit_byte(OpCode::Pop);
    }

    fn if_stmt(&mut self) {
        self.advance(); // Consume 'if'

        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let jump_then = self.emit_jump(OpCode::JumpIfFalse);

        // Then branch
        self.emit_byte(OpCode::Pop);
        self.statement();

        let jump_else = self.emit_jump(OpCode::Jump);

        // Else branch
        self.path_jump(jump_then);
        self.emit_byte(OpCode::Pop);

        if self._match(TokenKind::Else) {
            self.statement();
        }

        self.path_jump(jump_else);
    }

    fn block_stmt(&mut self) {
        self.advance(); // Consume '{'

        self.context.begin_scope();
        self.block();

        let pops = self.context.end_scope();
        self.emit_n_bytes(pops, OpCode::Pop as u8);
    }

    fn print_stmt(&mut self) {
        self.advance(); // Consume PRINT token

        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Print);
    }

    fn expression_stmt(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn block(&mut self) {
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::EOF) {
            self.declaration();
        }

        self.consume(TokenKind::RightBrace, "Expect '}' after block.");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assign);
    }

    fn number(&mut self) {
        let str = self.parser.prev.lexeme(self.source);
        // The scanner has the job of ensuring that the lexeme is a number
        let val = f64::from_str(str).unwrap();
        let const_ = self.make_constant(Constant::Number(val));

        self.emit_bytes(OpCode::Cons as u8, const_);
    }

    fn unary(&mut self) {
        let op = self.parser.prev.kind;
        self.parse_precedence(Precedence::Unary);

        match op {
            TokenKind::Minus => self.emit_byte(OpCode::Neg),
            TokenKind::Bang => self.emit_byte(OpCode::Not),
            _ => {}
        };
    }

    fn string(&mut self) {
        let const_ = self.make_constant(Constant::String {
            start: self.parser.prev.span.start + 1,
            end: self.parser.prev.span.end - 1,
        });

        self.emit_bytes(OpCode::Cons as u8, const_);
    }

    fn literal(&mut self) {
        match self.parser.prev.kind {
            TokenKind::False => self.emit_byte(OpCode::False),
            TokenKind::True => self.emit_byte(OpCode::True),
            TokenKind::Nil => self.emit_byte(OpCode::Nil),
            _ => {} // Unreachable
        }
    }

    fn or(&mut self) {
        let jump_left = self.emit_jump(OpCode::JumpIfFalse);

        let jump_right = self.emit_jump(OpCode::Jump);
        self.path_jump(jump_left);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::Or);

        self.path_jump(jump_right);
    }

    fn and(&mut self) {
        let jump_right = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.path_jump(jump_right);
    }

    fn binary(&mut self) {
        let op = self.parser.prev.kind;
        let precedence = self.parser_rule_from(&op).precedence;
        self.parse_precedence(precedence.next());

        match op {
            TokenKind::BangEqual => self.emit_bytes(OpCode::Eq, OpCode::Not),
            TokenKind::EqualEqual => self.emit_byte(OpCode::Eq),
            TokenKind::Greater => self.emit_byte(OpCode::Greater),
            TokenKind::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenKind::Less => self.emit_byte(OpCode::Less),
            TokenKind::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),

            TokenKind::Plus => self.emit_byte(OpCode::Add),
            TokenKind::Minus => self.emit_byte(OpCode::Sub),
            TokenKind::Star => self.emit_byte(OpCode::Mul),
            TokenKind::Slash => self.emit_byte(OpCode::Div),
            TokenKind::Percent => self.emit_byte(OpCode::Mod),
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

        let can_assign = prec <= Precedence::Assign;
        self.parser.can_assign = can_assign;

        self.compile_prefix(prefix);
        while prec <= self.parser_rule_from(&self.parser.curr.kind).precedence {
            self.advance();
            if let Some(infix) = self.parser_rule_from(&self.parser.prev.kind).infix {
                self.compile_infix(infix);
            };
        }

        if self.parser.can_assign && self._match(TokenKind::Equal) {
            self.error_at(self.parser.curr, "Invalid assignment target.");
        }
    }

    fn make_constant(&mut self, constant: Constant) -> Byte {
        let const_idx = self.chunk.add_const(constant);

        if const_idx > u8::MAX as usize {
            self.error("Too many constants in a chunk");

            0
        } else {
            const_idx as u8
        }
    }

    fn compile_prefix(&mut self, prefix: PrefixRule) {
        match prefix {
            PrefixRule::Grouping => self.grouping(),
            PrefixRule::Variable => self.variable(),
            PrefixRule::Number => self.number(),
            PrefixRule::Unary => self.unary(),
            PrefixRule::Literal => self.literal(),
            PrefixRule::String => self.string(),
        }
    }

    fn compile_infix(&mut self, infix: InfixRule) {
        match infix {
            InfixRule::Binary => self.binary(),
            InfixRule::And => self.and(),
            InfixRule::Or => self.or(),
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

    fn syncronize(&mut self) {
        self.parser.panic_mode = false;
        while !self._match(TokenKind::EOF) {
            match self.parser.prev.kind {
                TokenKind::Semicolon
                | TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => self.advance(),
            }
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

    fn check(&self, expected: TokenKind) -> bool {
        self.parser.curr.kind == expected
    }

    fn _match(&mut self, expected: TokenKind) -> bool {
        if self.check(expected) {
            self.advance();
            return true;
        }
        false
    }

    fn idents_equals(&self, a: Token, b: Token) -> bool {
        let a_str = &self.source[a.span.start..a.span.end];
        let b_str = &self.source[b.span.start..b.span.end];

        a_str == b_str
    }

    fn path_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 2;
        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        self.chunk.code[offset] = (jump >> 8) as u8;
        self.chunk.code[offset + 1] = jump as u8;
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop);
        let offset = self.chunk.code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        self.emit_byte((offset >> 8) as u8);
        self.emit_byte(offset as u8);
    }

    fn emit_jump(&mut self, jmp_kind: OpCode) -> usize {
        self.emit_byte(jmp_kind);
        self.emit_bytes(0xff, 0xff);

        self.chunk.code.len() - 2
    }

    fn emit_n_bytes(&mut self, n: usize, byte: Byte) {
        for _ in 0..n {
            self.emit_byte(byte);
        }
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
            TokenKind::Percent,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Factor),
        );

        self.rules.insert(
            TokenKind::Number,
            ParseRule::default().prefix(PrefixRule::Number),
        );
        self.rules.insert(
            TokenKind::True,
            ParseRule::default().prefix(PrefixRule::Literal),
        );
        self.rules.insert(
            TokenKind::False,
            ParseRule::default().prefix(PrefixRule::Literal),
        );
        self.rules.insert(
            TokenKind::Nil,
            ParseRule::default().prefix(PrefixRule::Literal),
        );
        self.rules.insert(
            TokenKind::Bang,
            ParseRule::default().prefix(PrefixRule::Unary),
        );

        self.rules.insert(
            TokenKind::BangEqual,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Equality),
        );
        self.rules.insert(
            TokenKind::EqualEqual,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Equality),
        );
        self.rules.insert(
            TokenKind::Greater,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Comparison),
        );
        self.rules.insert(
            TokenKind::GreaterEqual,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Comparison),
        );
        self.rules.insert(
            TokenKind::Less,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Comparison),
        );
        self.rules.insert(
            TokenKind::LessEqual,
            ParseRule::default()
                .infix(InfixRule::Binary)
                .precedence(Precedence::Comparison),
        );

        self.rules.insert(
            TokenKind::String,
            ParseRule::default().prefix(PrefixRule::String),
        );

        self.rules.insert(
            TokenKind::Identifier,
            ParseRule::default().prefix(PrefixRule::Variable),
        );

        self.rules.insert(
            TokenKind::And,
            ParseRule::default()
                .infix(InfixRule::And)
                .precedence(Precedence::And),
        );
        self.rules.insert(
            TokenKind::Or,
            ParseRule::default()
                .infix(InfixRule::Or)
                .precedence(Precedence::Or),
        );

        self.rules.insert(TokenKind::EOF, ParseRule::default());
    }
}

impl Precedence {
    fn next(self) -> Self {
        match self {
            Precedence::None => Precedence::Assign,
            Precedence::Assign => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::_Call,
            Precedence::_Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}
