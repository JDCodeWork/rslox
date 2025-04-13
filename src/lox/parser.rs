use crate::errors::LoxError;

use super::{
    expr::{Binary, Expr, Grouping, Literal, Unary},
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
    pub fn parse(&mut self) -> Result<Expr, LoxError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expression = self.comparison()?;

        while self.match_token(&[BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expression = Expr::Binary(Binary::new(expression, operator, right));
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        let mut expression = self.term()?;

        while self.match_token(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expression = Expr::Binary(Binary::new(expression, operator, right))
        }

        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
        let mut expression = self.factor()?;

        while self.match_token(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            expression = Expr::Binary(Binary::new(expression, operator, right))
        }

        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
        let mut expression = self.unary()?;

        while self.match_token(&[Star, Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expression = Expr::Binary(Binary::new(expression, operator, right))
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.match_token(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            Ok(Expr::Unary(Unary::new(operator, right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        let expression = match self.peek().get_type() {
            False => {
                self.advance();
                Expr::Literal(Literal::new("false".to_string()))
            }
            True => {
                self.advance();
                Expr::Literal(Literal::new("true".to_string()))
            }
            Nil => {
                self.advance();
                Expr::Literal(Literal::new("null".to_string()))
            }
            Number | String => {
                let lit = self.peek().get_literal().to_string();
                self.advance();
                Expr::Literal(Literal::new(lit))
            }
            LeftParen => {
                self.advance();
                let expr = self.expression()?;

                self.consume(RightParen, "Expect ')' after expression.")?;

                Expr::Grouping(Grouping::new(expr))
            }
            _ => return Err(LoxError::UnknownType(self.current)),
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

    fn consume(&mut self, token_type: TokenType, error: &str) -> Result<(), LoxError> {
        if self.check(&token_type) {
            self.advance();
            return Ok(());
        };

        Err(LoxError::CustomError(self.current, error.to_string()))
    }
}
