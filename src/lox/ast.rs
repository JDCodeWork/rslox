use crate::tools::AstPrinter;

use super::token::Token;

// region: higher-level structures

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(VarStmt),
    If(IfStmt),
    Block(Vec<Stmt>),
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

// endregion

// region: lower-level structures

// region: Stmt structures
#[derive(Debug, PartialEq, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_b: Box<Stmt>,
    pub else_b: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarStmt {
    pub name: Token,
    pub val: Expr,
}

// endregion

// region: Expr structures
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

// endregion

// endregion

// region: Into trait implementation

impl Into<Stmt> for IfStmt {
    fn into(self) -> Stmt {
        Stmt::If(self)
    }
}

impl Into<Stmt> for VarStmt {
    fn into(self) -> Stmt {
        Stmt::Var(self)
    }
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

impl Into<Stmt> for Literal {
    fn into(self) -> Stmt {
        Stmt::Expression(self.into())
    }
}

impl Into<Expr> for Token {
    fn into(self) -> Expr {
        Expr::Var(self)
    }
}

// endregion

// region: Implementation of new associated function
impl IfStmt {
    pub fn new(cond: Expr, then_b: Stmt, else_b: Stmt) -> Self {
        Self {
            condition: cond,
            then_b: Box::new(then_b),
            else_b: Box::new(else_b),
        }
    }
}

impl VarStmt {
    pub fn new(name: Token, val: Expr) -> Self {
        Self { name, val }
    }
}

impl Assignment {
    pub fn new(name: Token, initializer: Expr) -> Self {
        Self {
            name,
            value: Box::new(initializer),
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

// endregion

// region: implementation of printing for ast structures
impl Stmt {
    pub fn print(self) -> String {
        match self {
            Stmt::Expression(expr) => expr.print(),
            Stmt::Print(expr) => format!("(print {})", expr.print()),
            Stmt::Var(var_stmt) => {
                format!(
                    "(var {} = {})",
                    var_stmt.name.to_string(),
                    var_stmt.val.print()
                )
            }
            Stmt::Block(stmts) => {
                let mut result = String::from("(block");
                for stmt in stmts {
                    result.push_str(&format!(" {}", stmt.print()));
                }
                result.push(')');
                result
            }
            Stmt::If(if_stmt) => {
                let IfStmt {
                    condition,
                    then_b,
                    else_b,
                } = if_stmt;

                format!(
                    "(if {} then {} else {})",
                    condition.print(),
                    then_b.print(),
                    else_b.print()
                )
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

// endregion
