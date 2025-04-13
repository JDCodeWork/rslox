use crate::tools::AstPrinter;

use super::token::Token;

pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

impl Expr {
    pub fn print(self) -> String {
        match self {
            Expr::Binary(binary) => {
                let Binary {
                    left,
                    operator,
                    right,
                } = binary;

                AstPrinter::parenthesize(&operator.get_lexeme(), vec![left, right])
            }
            Expr::Grouping(group) => AstPrinter::parenthesize("group", vec![group.expression]),
            Expr::Literal(literal) => {
                if literal.value == "null" {
                    return "nill".to_string();
                }

                literal.value.to_string()
            }
            Expr::Unary(unary) => {
                let Unary { operator, right } = unary;

                AstPrinter::parenthesize(&operator.get_lexeme(), vec![right])
            }
        }
    }
}

pub struct Binary {
    left: Box<Expr>,
    operator: Token,
    right: Box<Expr>,
}

pub struct Grouping {
    expression: Box<Expr>,
}

pub struct Literal {
    value: String,
}

pub struct Unary {
    operator: Token,
    right: Box<Expr>,
}

impl Binary {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

impl Grouping {
    pub fn new(expression: Expr) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}

impl Literal {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Unary {
    pub fn new(operator: Token, right: Expr) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}
