use crate::{
    errors::{Err, ParseErr, RuntimeErr},
    lox::ast::{Assignment, Stmt, VarStmt},
};

use super::{
    ast::{Binary, Expr, Grouping, Literal, Unary},
    token::{
        Token,
        TokenType::{self, *},
    },
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
}

impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Err> {
        let mut staments = Vec::new();

        while !self.is_at_end() {
            staments.push(self.declaration()?);
        }

        Ok(staments)
    }

    fn declaration(&mut self) -> Result<Stmt, Err> {
        let stmt = match *self.peek().get_type() {
            Var => self.var_dec(),
            _ => self.statment(),
        };

        if let Err(lox_err) = stmt {
            self.synchronize();
            lox_err.report_and_exit(1);
        }

        stmt
    }

    fn var_dec(&mut self) -> Result<Stmt, Err> {
        self.advance();
        let name = self.consume(Identifier, "Expected a variable name")?;

        let mut initialicer = Literal::Nil.into();
        if self.match_token(&[Equal]) {
            initialicer = self.expression()?;
        }
        self.consume(Semicolon, "Expect ';' after variable declaration.")?;

        Ok(VarStmt::new(name, initialicer).into())
    }

    fn statment(&mut self) -> Result<Stmt, Err> {
        match *self.peek().get_type() {
            Print => self.print_stmt(),
            LeftBrace => self.block_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn block_stmt(&mut self) -> Result<Stmt, Err> {
        self.advance(); // Consume '{' token

        let mut stmts = Vec::new();

        while !self.check(&RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(RightBrace, "Expected '}' after block.")?;

        Ok(Stmt::Block(stmts))
    }

    fn print_stmt(&mut self) -> Result<Stmt, Err> {
        self.advance(); // Consume 'Print' token

        let val = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value.")?;

        Ok(Stmt::Print(val))
    }

    fn expr_stmt(&mut self) -> Result<Stmt, Err> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expected ';' after expression.")?;

        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, Err> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Err> {
        let expr = self.equality()?;

        if !self.match_token(&[Equal]) {
            return Ok(expr);
        }

        let val = self.assignment()?;

        if let Expr::Var(name) = expr {
            Ok(Assignment::new(name, val).into())
        } else {
            Err(RuntimeErr::InvalidAssigment.to_err())
        }
    }

    fn equality(&mut self) -> Result<Expr, Err> {
        let mut expression = self.comparison()?;

        while self.match_token(&[BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expression = Binary::new(expression, operator, right).into();
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expr, Err> {
        let mut expression = self.term()?;

        while self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expression = Binary::new(expression, operator, right).into()
        }

        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr, Err> {
        let mut expression = self.factor()?;

        while self.match_token(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expression = Binary::new(expression, operator, right).into()
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr, Err> {
        let mut expression = self.unary()?;

        while self.match_token(&[Star, Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expression = Binary::new(expression, operator, right).into()
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr, Err> {
        if self.match_token(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            Ok(Unary::new(operator, right).into())
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, Err> {
        let token_type = self.peek().get_type().clone();

        let expression = match token_type {
            False => {
                self.advance();
                Literal::Boolean(false).into()
            }
            True => {
                self.advance();
                Literal::Boolean(true).into()
            }
            Nil => {
                self.advance();
                Literal::Nil.into()
            }
            Number(num) => {
                self.advance();
                Literal::Number(num).into()
            }
            String(str) => {
                self.advance();
                Literal::String(str).into()
            }
            LeftParen => {
                self.advance();
                let expr = self.expression()?;

                self.consume(RightParen, "Expect ')' after expression.")?;

                Grouping::new(expr).into()
            }
            Identifier => Expr::Var(self.advance().clone()),
            _ => return Err(ParseErr::UnexpectedEOF(self.current).to_err()),
        };
        Ok(expression)
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&mut self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().get_type() == token_type
        }
    }

    fn peek(&self) -> &Token {
        match self.tokens.get(self.current) {
            None => self.previous(),
            Some(token) => token,
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn previous(&self) -> &Token {
        if self.current == 0 {
            self.peek()
        } else {
            match self.tokens.get(self.current - 1) {
                None => self.peek(),
                Some(token) => token,
            }
        }
    }

    fn is_at_end(&self) -> bool {
        *self.peek().get_type() == EOF
    }

    fn consume(&mut self, token_type: TokenType, error: &str) -> Result<Token, Err> {
        if self.check(&token_type) {
            return Ok(self.advance().clone());
        };

        Err(ParseErr::ExpectedToken(error.to_string(), self.current).to_err())
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().get_type() == &Semicolon {
                return;
            }

            match *self.peek().get_type() {
                Class | Fun | Var | For | If | While | Print | Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}
