use crate::tools::AstPrinter;

use super::token::Token;

#[derive(PartialEq, Debug)]
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
            Expr::Literal(val) => match val {
                Literal::Nil => "nil".to_string(),
                Literal::Boolean(bool) => bool.to_string(),
                Literal::Number(num) => num.to_string(),
                Literal::String(str) => str.to_string(),
            },
            Expr::Unary(unary) => {
                let Unary { operator, right } = unary;

                AstPrinter::parenthesize(&operator.get_lexeme(), vec![right])
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
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

impl Unary {
    pub fn new(operator: Token, right: Expr) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}
