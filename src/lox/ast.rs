use crate::tools::AstPrinter;

use super::token::Token;

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    // Name, Initializer
    Var(Token, Expr),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Assign(Assignment),
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Var(Token),
}

impl Stmt {
    pub fn print(self) -> String {
        match self {
            Stmt::Expression(expr) => expr.print(),
            Stmt::Print(expr) => format!("(print {})", expr.print()),
            Self::Var(name, initializer) => {
                format!("(var {} = {})", name.to_string(), initializer.print())
            }
        }
    }
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
            Expr::Var(str) => {
                format!("var {str}")
            }
            Expr::Assign(assign) => format!(
                "Assign {} to {}",
                assign.value.print(),
                assign.name.get_lexeme()
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Into<Expr> for Assignment {
    fn into(self) -> Expr {
        Expr::Assign(self)
    }
}

impl Into<Expr> for Binary {
    fn into(self) -> Expr {
        Expr::Binary(self)
    }
}

impl Into<Expr> for Grouping {
    fn into(self) -> Expr {
        Expr::Grouping(self)
    }
}

impl Into<Expr> for Unary {
    fn into(self) -> Expr {
        Expr::Unary(self)
    }
}

impl Into<Expr> for Literal {
    fn into(self) -> Expr {
        Expr::Literal(self)
    }
}

impl Into<Expr> for Token {
    fn into(self) -> Expr {
        Expr::Var(self)
    }
}

impl Assignment {
    pub fn new(name: Token, initialicer: Expr) -> Self {
        Self {
            name,
            value: Box::new(initialicer),
        }
    }
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
