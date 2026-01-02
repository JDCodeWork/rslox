use crate::{
    errors::{Err, ParseErr, RuntimeErr},
    lox::ast::{
        AssignmentExpr, CallExpr, ClassStmt, FunStmt, GetExpr, IfStmt, LogicalExpr, ReturnStmt,
        Stmt, VarExpr, VarStmt, WhileStmt,
    },
};

use super::{
    ast::{BinaryExpr, Expr, GroupingExpr, LiteralExpr, UnaryExpr},
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
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, Err> {
        let stmt = match *self.peek().get_type() {
            Class => self.class_dec(),
            Fun => {
                self.advance(); // Consume FUN token
                self.fun_dec("function")
            }
            Var => {
                self.advance();
                self.var_dec()
            }
            _ => self.statement(),
        };

        if let Err(lox_err) = stmt {
            self.synchronize();
            lox_err.report_and_exit(1);
        }

        stmt
    }

    fn class_dec(&mut self) -> Result<Stmt, Err> {
        self.advance(); // Consume CLASS token

        let name = self.consume(Identifier, "Expect class name")?;
        self.consume(LeftBrace, "Expected '{' after class name.")?;

        let mut methods: Vec<FunStmt> = Vec::new();
        while !self.check(&RightBrace) && !self.is_at_end() {
            if let Stmt::Function(fun) = self.fun_dec("method")? {
                methods.push(fun);
            }
        }
        self.consume(RightBrace, "Expected '}' after class body.")?;

        let class = ClassStmt::new(name, methods);
        Ok(class.into())
    }

    fn fun_dec(&mut self, kind: &str) -> Result<Stmt, Err> {
        let name = self.consume(Identifier, format!("Expect {kind} name").as_str())?;

        self.consume(
            LeftParen,
            format!("Expected '(' after {kind} name.").as_str(),
        )?;
        let mut params = Vec::new();

        // handles the zero parameters case
        if !self.check(&RightParen) {
            loop {
                if params.len() >= 255 {
                    ParseErr::TooManyArguments(kind.to_string(), name.get_line())
                        .into_err()
                        .report();
                }

                params.push(self.consume(Identifier, "Expect parameter name.")?);

                if !self.match_token(&[Comma]) {
                    break;
                }
            }
        }
        self.consume(RightParen, "Expect ')' after parameters.")?;

        let body = self.block_stmt()?;

        Ok(FunStmt::new(name, params, body, None).into())
    }

    fn var_dec(&mut self) -> Result<Stmt, Err> {
        let name = self.consume(Identifier, "Expected a variable name")?;

        let mut init = LiteralExpr::Nil.into();
        if self.match_token(&[Equal]) {
            init = self.expression()?;
        }
        self.consume(Semicolon, "Expect ';' after variable declaration.")?;

        Ok(VarStmt::new(name, init).into())
    }

    fn statement(&mut self) -> Result<Stmt, Err> {
        if Print == *self.peek().get_type() {
            self.peek();
        }

        match *self.peek().get_type() {
            Print => self.print_stmt(),
            LeftBrace => self.block_stmt(),
            If => self.if_stmt(),
            While => self.while_stmt(),
            For => self.for_stmt(),
            Return => self.return_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn return_stmt(&mut self) -> Result<Stmt, Err> {
        let keyword = self.advance().clone();

        let mut val: Expr = LiteralExpr::Nil.into();
        if !self.check(&Semicolon) {
            val = self.expression()?;
        }
        self.consume(Semicolon, "Expect ';' after return value.")?;

        Ok(ReturnStmt::new(keyword, val).into())
    }

    fn while_stmt(&mut self) -> Result<Stmt, Err> {
        self.advance(); // Consume 'while'
        self.consume(LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after condition.")?;

        let body = self.statement()?;

        Ok(WhileStmt::new(condition, body).into())
    }

    fn for_stmt(&mut self) -> Result<Stmt, Err> {
        self.advance();

        self.consume(LeftParen, "Expect '(' after 'for'.")?;

        let initializer: Stmt;
        if self.match_token(&[Semicolon]) {
            initializer = LiteralExpr::Nil.into();
        } else if self.match_token(&[Var]) {
            initializer = self.var_dec()?;
        } else {
            initializer = self.expression()?.into();
        }

        let condition: Expr;
        if self.match_token(&[Semicolon]) {
            condition = LiteralExpr::Boolean(true).into();
        } else {
            condition = self.expression()?;
        }
        self.consume(Semicolon, "Expect ';' after a loop condition.")?;

        let increment: Stmt;
        if self.match_token(&[RightParen]) {
            increment = LiteralExpr::Nil.into();
        } else {
            increment = self.expression()?.into();
        }
        self.consume(RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if increment != LiteralExpr::Nil.into() {
            body = Stmt::Block(vec![body, increment]);
        }

        let mut stmt = WhileStmt::new(condition, body).into();
        if initializer != LiteralExpr::Nil.into() {
            stmt = Stmt::Block(vec![initializer, stmt]);
        }

        Ok(stmt)
    }

    fn if_stmt(&mut self) -> Result<Stmt, Err> {
        self.advance(); // Consume If token

        self.advance(); // Consume '(' token
        let cond = self.expression()?;
        self.advance(); // Consume ')' token

        let then_b = self.statement()?;

        let mut else_b: Stmt = LiteralExpr::Nil.into();
        if self.match_token(&[Else]) {
            else_b = self.statement()?;
        }

        Ok(IfStmt::new(cond, then_b, else_b).into())
    }

    fn block_stmt(&mut self) -> Result<Stmt, Err> {
        self.consume(LeftBrace, "Expect '{' before block.")?;

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
        let expr = self.logic_or()?;

        if !self.match_token(&[Equal]) {
            return Ok(expr);
        }

        let val = self.assignment()?;

        if let Expr::Var(var_expr) = expr {
            Ok(AssignmentExpr::new(var_expr.name, val).into())
        } else {
            Err(RuntimeErr::InvalidAssignment.to_err())
        }
    }

    fn logic_or(&mut self) -> Result<Expr, Err> {
        let mut expr = self.logic_and()?;

        while self.match_token(&[Or]) {
            let op = self.previous().clone();
            let right = self.logic_and()?;

            expr = LogicalExpr::new(expr, op, right).into();
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, Err> {
        let mut expr = self.equality()?;

        while self.match_token(&[And]) {
            let op = self.previous().clone();
            let right = self.equality()?;

            expr = LogicalExpr::new(expr, op, right).into();
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Err> {
        let mut expression = self.comparison()?;

        while self.match_token(&[BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expression = BinaryExpr::new(expression, operator, right).into();
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expr, Err> {
        let mut expression = self.term()?;

        while self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expression = BinaryExpr::new(expression, operator, right).into()
        }

        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr, Err> {
        let mut expression = self.factor()?;

        while self.match_token(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expression = BinaryExpr::new(expression, operator, right).into()
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr, Err> {
        let mut expression = self.unary()?;

        while self.match_token(&[Star, Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expression = BinaryExpr::new(expression, operator, right).into()
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr, Err> {
        if self.match_token(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            Ok(UnaryExpr::new(operator, right).into())
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, Err> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[Dot]) {
                let name = self.consume(Identifier, "Expect property name after '.'.")?;
                expr = GetExpr::new(expr, name).into();
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Err> {
        let mut args = Vec::new();

        if !self.check(&RightParen) {
            loop {
                if args.len() >= 255 {
                    ParseErr::TooManyArguments(callee.clone().print(), self.peek().get_line())
                        .into_err()
                        .report();
                }

                args.push(self.expression()?);

                if !self.match_token(&[Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(RightParen, "Expected ')' after arguments.")?;

        let call_expr = CallExpr::new(callee, paren, args);

        Ok(call_expr.into())
    }

    fn primary(&mut self) -> Result<Expr, Err> {
        let token_type = self.peek().get_type().clone();

        let expression = match token_type {
            False => {
                self.advance();
                LiteralExpr::Boolean(false).into()
            }
            True => {
                self.advance();
                LiteralExpr::Boolean(true).into()
            }
            Nil => {
                self.advance();
                LiteralExpr::Nil.into()
            }
            Number(num) => {
                self.advance();
                LiteralExpr::Number(num).into()
            }
            String(str) => {
                self.advance();
                LiteralExpr::String(str).into()
            }
            LeftParen => {
                self.advance();
                let expr = self.expression()?;

                self.consume(RightParen, "Expect ')' after expression.")?;

                GroupingExpr::new(expr).into()
            }
            Identifier => VarExpr::new(self.advance().clone()).into(),
            _ => return Err(ParseErr::UnexpectedEOF(self.current).into_err()),
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

        Err(ParseErr::ExpectedToken(error.to_string(), self.current).into_err())
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
