use super::{
    expr::{Binary, Expr},
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
    #[allow(unused)]
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expression = self.comparison();

        while self.match_token(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();

            expression = Expr::Binary(Binary::new(expression, operator, right));
        }

        expression
    }

    fn comparison(&mut self) -> Expr {
        unimplemented!()
    }

    fn previous(&mut self) -> Token {
        unimplemented!()
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
        unimplemented!()
    }

    fn advance(&mut self) {
        unimplemented!()
    }

    fn is_at_end(&self) -> bool {
        unimplemented!()
    }
}
