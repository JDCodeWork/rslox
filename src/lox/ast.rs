use crate::{
    errors::Err,
    lox::{env::EnvId, interpreter::Interpreter},
    tools::AstPrinter,
};

use super::token::Token;

// region: higher-level structures

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(VarStmt),
    If(IfStmt),
    While(WhileStmt),
    Function(FunStmt),
    Block(Vec<Stmt>),
    Return(ReturnStmt),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Assign(AssignmentExpr),
    Binary(BinaryExpr),
    Logical(LogicalExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Var(Token),
    Call(CallExpr),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Callable {
    User(FunStmt),
    Native(NativeFn),
}

#[derive(PartialEq, Debug, Clone)]
pub enum BlockKind {
    Default,
    Closure,
}

// endregion

// region: lower-level structures

// region: Stmt structures

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Expr,
}

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

#[derive(Debug, PartialEq, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

// endregion

// region: Expr structures

#[derive(Debug, PartialEq, Clone)]
pub struct AssignmentExpr {
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunStmt {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Box<Stmt>,
    pub closure: Option<EnvId>,
}

#[derive(Debug, Clone)]
pub struct NativeFn {
    pub arity: u8,
    pub action: fn(&mut Interpreter, Vec<LiteralExpr>) -> Result<LiteralExpr, Err>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub args: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LogicalExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExpr {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Call(Callable),
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

// endregion

// endregion

// region: Into trait implementation

impl Into<Stmt> for ReturnStmt {
    fn into(self) -> Stmt {
        Stmt::Return(self)
    }
}

impl Into<Stmt> for FunStmt {
    fn into(self) -> Stmt {
        Stmt::Function(self)
    }
}

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

impl Into<Stmt> for WhileStmt {
    fn into(self) -> Stmt {
        Stmt::While(self)
    }
}
impl Into<Stmt> for Expr {
    fn into(self) -> Stmt {
        Stmt::Expression(self)
    }
}

impl Into<Expr> for CallExpr {
    fn into(self) -> Expr {
        Expr::Call(self)
    }
}

impl Into<Expr> for AssignmentExpr {
    fn into(self) -> Expr {
        Expr::Assign(self)
    }
}

impl Into<Expr> for BinaryExpr {
    fn into(self) -> Expr {
        Expr::Binary(self)
    }
}

impl Into<Expr> for LogicalExpr {
    fn into(self) -> Expr {
        Expr::Logical(self)
    }
}

impl Into<Expr> for GroupingExpr {
    fn into(self) -> Expr {
        Expr::Grouping(self)
    }
}

impl Into<Expr> for UnaryExpr {
    fn into(self) -> Expr {
        Expr::Unary(self)
    }
}

impl Into<Expr> for LiteralExpr {
    fn into(self) -> Expr {
        Expr::Literal(self)
    }
}

impl Into<Stmt> for LiteralExpr {
    fn into(self) -> Stmt {
        Stmt::Expression(self.into())
    }
}

impl Into<Callable> for FunStmt {
    fn into(self) -> Callable {
        Callable::User(self)
    }
}

impl Into<LiteralExpr> for Callable {
    fn into(self) -> LiteralExpr {
        LiteralExpr::Call(self)
    }
}

impl Into<LiteralExpr> for NativeFn {
    fn into(self) -> LiteralExpr {
        LiteralExpr::Call(Callable::Native(self))
    }
}

impl Into<Expr> for Token {
    fn into(self) -> Expr {
        Expr::Var(self)
    }
}

// endregion

// region: Implementation of new associated function

impl ReturnStmt {
    pub fn new(keyword: Token, value: Expr) -> Self {
        Self { keyword, value }
    }
}

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

impl WhileStmt {
    pub fn new(cond: Expr, body: Stmt) -> Self {
        Self {
            condition: cond,
            body: Box::new(body),
        }
    }
}

impl FunStmt {
    pub fn new(name: Token, params: Vec<Token>, body: Stmt, closure: Option<EnvId>) -> Self {
        Self {
            name,
            params,
            body: Box::new(body),
            closure,
        }
    }
}

impl NativeFn {
    pub fn new(
        arity: u8,
        action: fn(&mut Interpreter, Vec<LiteralExpr>) -> Result<LiteralExpr, Err>,
    ) -> Self {
        Self { arity, action }
    }
}

impl CallExpr {
    pub fn new(callee: Expr, paren: Token, args: Vec<Expr>) -> Self {
        Self {
            callee: Box::new(callee),
            paren,
            args,
        }
    }
}

impl AssignmentExpr {
    pub fn new(name: Token, initializer: Expr) -> Self {
        Self {
            name,
            value: Box::new(initializer),
        }
    }
}

impl BinaryExpr {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

impl LogicalExpr {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

impl GroupingExpr {
    pub fn new(expression: Expr) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}

impl UnaryExpr {
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
            Stmt::Return(return_stmt) => {
                format!("(return {})", return_stmt.value.print())
            }
            Stmt::Expression(expr) => expr.print(),
            Stmt::Print(expr) => format!("(print {})", expr.print()),
            Stmt::Var(var_stmt) => {
                format!(
                    "(var {} = {})",
                    var_stmt.name.to_string(),
                    var_stmt.val.print()
                )
            }
            Stmt::Function(fn_stmt) => {
                format!(
                    "(fn {} ({}) {{}})",
                    fn_stmt.name.get_lexeme(),
                    fn_stmt
                        .params
                        .iter()
                        .map(|p| p.get_lexeme())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Stmt::While(while_stmt) => {
                format!(
                    "(while {} = {})",
                    while_stmt.condition.print(),
                    while_stmt.body.print()
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

impl Callable {
    pub fn print(self) -> String {
        match self {
            Callable::User(func) => Stmt::Function(func).print(),
            Callable::Native(_) => "<native>()".to_string(),
        }
    }
}
impl Expr {
    pub fn print(self) -> String {
        match self {
            Expr::Call(call_expr) => {
                let CallExpr {
                    callee,
                    paren: _,
                    args,
                } = call_expr;

                // Print callee concisely: if it's a simple variable, use its lexeme;
                // otherwise use the expression's print but strip a leading "call "
                let callee_repr = match *callee {
                    Expr::Var(token) => token.get_lexeme().to_string(),
                    other => {
                        let s = other.print();
                        // strip a leading "call " that nested call printing may add
                        if let Some(stripped) = s.strip_prefix("call ") {
                            stripped.to_string()
                        } else {
                            s
                        }
                    }
                };

                let printed_args: Vec<String> = args.into_iter().map(|arg| arg.print()).collect();
                let args = printed_args.join(", ");
                if args.is_empty() {
                    format!("call {}()", callee_repr)
                } else {
                    format!("call {}({})", callee_repr, args)
                }
            }
            Expr::Binary(binary) => {
                let BinaryExpr {
                    left,
                    operator,
                    right,
                } = binary;

                AstPrinter::parenthesize(&operator.get_lexeme(), vec![left, right])
            }
            Expr::Logical(logical) => {
                let LogicalExpr {
                    left,
                    operator,
                    right,
                } = logical;

                AstPrinter::parenthesize(&operator.get_lexeme(), vec![left, right])
            }
            Expr::Grouping(group) => AstPrinter::parenthesize("group", vec![group.expression]),
            Expr::Literal(val) => match val {
                LiteralExpr::Nil => "nil".to_string(),
                LiteralExpr::Boolean(bool) => bool.to_string(),
                LiteralExpr::Number(num) => num.to_string(),
                LiteralExpr::String(str) => str.to_string(),
                LiteralExpr::Call(call_expr) => call_expr.print(),
            },
            Expr::Unary(unary) => {
                let UnaryExpr { operator, right } = unary;

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

impl PartialEq for NativeFn {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity
    }
}
