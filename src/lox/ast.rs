use std::collections::HashMap;
use std::fmt;

use crate::{
    errors::LoxError,
    lox::{env::EnvId, interpreter::Interpreter},
    tools::AstPrinter,
};

use super::token::Token;

// region: Macros

macro_rules! impl_into {
    ($from:ty, $to:ty, $variant:path) => {
        impl Into<$to> for $from {
            fn into(self) -> $to {
                $variant(self)
            }
        }
    };
    ($group:ty; $($from:ty => $to:path),+ $(,)?) => {
        $( impl_into!($from, $group, $to); )*
    };
}

macro_rules! impl_new {
    // Pattern for custom field creation (e.g. boxing)
    ($type:ty, ($($arg:ident : $arg_type:ty),*), { $($fields:tt)* }) => {
        impl $type {
            pub fn new($($arg : $arg_type),*) -> Self {
                Self {
                    $($fields)*
                }
            }
        }
    };
    // Pattern for simple field mapping (arg name matches field name)
    ($type:ty, ($($arg:ident : $arg_type:ty),*)) => {
        impl $type {
            pub fn new($($arg : $arg_type),*) -> Self {
                Self {
                    $($arg),*
                }
            }
        }
    };
}

// endregion: Macros

// region: AST Enums

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
    Class(ClassStmt),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Assign(AssignmentExpr),
    Binary(BinaryExpr),
    Logical(LogicalExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    This(ThisExpr),
    Var(VarExpr),
    Call(CallExpr),
    Get(GetExpr),
    Set(SetExpr),
    Super(SuperExpr),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Callable {
    User(FunStmt),
    Class(ClassDec),
    Native(NativeFn),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    Callable(Callable),
    Instance(ClassInstance),
}

// endregion: AST Enums

// region: AST Nodes

// region: Statements

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

#[derive(Debug, PartialEq, Clone)]
pub struct ClassStmt {
    pub name: Token,
    pub superclass: Option<VarExpr>,
    pub methods: Vec<FunStmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDec {
    pub name: String,
    pub superclass: Option<Box<ClassDec>>,
    pub methods: HashMap<String, FunStmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunStmt {
    pub is_init: bool,
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Box<Stmt>,
    pub closure: Option<EnvId>,
}

#[derive(Debug, Clone)]
pub struct NativeFn {
    pub arity: u8,
    pub action: fn(&mut Interpreter, Vec<LiteralExpr>) -> Result<LiteralExpr, LoxError>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassInstance {
    pub id: usize,
    pub dec: ClassDec,
    pub fields: HashMap<String, LiteralExpr>,
}

// endregion: Statements

// region: Expressions

#[derive(Debug, PartialEq, Clone)]
pub struct SuperExpr {
    pub keyword: Token,
    pub method: Token,
    pub depth: Option<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ThisExpr {
    pub keyword: Token,
    pub depth: Option<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetExpr {
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GetExpr {
    pub object: Box<Expr>,
    pub name: Token,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssignmentExpr {
    pub name: Token,
    pub value: Box<Expr>,
    pub depth: Option<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarExpr {
    pub name: Token,
    pub depth: Option<usize>, // Depth in the environment stack; None means global
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
    Call(usize),
    Instance(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

// endregion: Expressions

// endregion: AST Nodes

// region: Traits: Into
impl_into!(Stmt;
    ClassStmt => Stmt::Class,
    ReturnStmt => Stmt::Return,
    FunStmt => Stmt::Function,
    IfStmt => Stmt::If,
    VarStmt => Stmt::Var,
    WhileStmt => Stmt::While,
    Expr => Stmt::Expression
);

impl_into!(Expr;
    ThisExpr => Expr::This,
    SetExpr => Expr::Set,
    GetExpr => Expr::Get,
    CallExpr => Expr::Call,
    VarExpr => Expr::Var,
    AssignmentExpr => Expr::Assign,
    BinaryExpr => Expr::Binary,
    LogicalExpr => Expr::Logical,
    GroupingExpr => Expr::Grouping,
    UnaryExpr => Expr::Unary,
    LiteralExpr => Expr::Literal,
    SuperExpr => Expr::Super
);

impl Into<Stmt> for LiteralExpr {
    fn into(self) -> Stmt {
        Stmt::Expression(self.into())
    }
}

impl Into<Object> for FunStmt {
    fn into(self) -> Object {
        Object::Callable(Callable::User(self))
    }
}

impl Into<Object> for NativeFn {
    fn into(self) -> Object {
        Object::Callable(Callable::Native(self))
    }
}

impl Into<Object> for ClassDec {
    fn into(self) -> Object {
        Object::Callable(Callable::Class(self))
    }
}

impl_into!(ClassInstance, Object, Object::Instance);

// endregion: Traits: Into

// region: Constructors

impl_new!(SuperExpr, (keyword: Token, method: Token), {
    method,
    keyword,
    depth: None
});

impl_new!(ClassDec, (name: String, methods: HashMap<String, FunStmt>, superclass: Option<ClassDec> ), {
    methods,
    name,
    superclass: superclass.map_or(None, |s| Some(Box::new(s)))
} );

impl_new!(ClassStmt, (name: Token, methods: Vec<FunStmt>, superclass: Option<VarExpr>) );

impl_new!(ReturnStmt, (keyword: Token, value: Expr) );

impl_new!(VarStmt, (name: Token, val: Expr) );

impl_new!(NativeFn, (
    arity: u8,
    action: fn(&mut Interpreter, Vec<LiteralExpr>) -> Result<LiteralExpr, LoxError>
));

impl_new!(IfStmt, (condition: Expr, then_b: Stmt, else_b: Stmt), {
    condition,
    then_b: Box::new(then_b),
    else_b: Box::new(else_b)
});

impl_new!(WhileStmt, (condition: Expr, body: Stmt), {
    condition,
    body: Box::new(body)
});

impl_new!(FunStmt, (name: Token, params: Vec<Token>, body: Stmt, is_init: bool), {
    name,
    params,
    is_init,
    closure: None,
    body: Box::new(body),
});

impl_new!(ThisExpr, (keyword: Token), {
    keyword,
    depth: None,
} );

impl_new!(SetExpr, (object: Expr, name: Token, value: Expr), {
    name,
    object: Box::new(object),
    value: Box::new(value),
} );

impl_new!(GetExpr, (object: Expr, name: Token), {
    object: Box::new(object),
    name,
} );

impl_new!(CallExpr, (callee: Expr, paren: Token, args: Vec<Expr>), {
    callee: Box::new(callee),
    paren,
    args,
});

impl_new!(AssignmentExpr, (name: Token, initializer: Expr), {
    name,
    value: Box::new(initializer),
    depth: None,
});

impl_new!(VarExpr, (name: Token), {
    name,
    depth: None
});

impl_new!(BinaryExpr, (left: Expr, operator: Token, right: Expr), {
    operator,
    left: Box::new(left),
    right: Box::new(right),
});

impl_new!(ClassInstance, (dec: ClassDec, id: usize), {
    dec,
    fields: HashMap::new(),
    id
});

impl_new!(LogicalExpr, (left: Expr, operator: Token, right: Expr), {
    operator,
    left: Box::new(left),
    right: Box::new(right),
});

impl_new!(GroupingExpr, (expression: Expr), {
    expression: Box::new(expression),
});

impl_new!(UnaryExpr, (operator: Token, right: Expr), {
    operator,
    right: Box::new(right),
});

// endregion: Constructors

// region: Formatting & Display
impl Stmt {
    pub fn print(self) -> String {
        match self {
            Self::Class(class) => {
                format!("(class {})", class.name.lexeme)
            }
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
                    fn_stmt.name.lexeme,
                    fn_stmt
                        .params
                        .iter()
                        .map(|p| p.lexeme.clone())
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

impl Expr {
    pub fn print(self) -> String {
        match self {
            Expr::Super(super_expr) => format!("(super {})", super_expr.method.lexeme),
            Expr::This(this_expr) => format!("(this {})", this_expr.keyword.line),
            Expr::Set(set_expr) => {
                format!("(set {})", set_expr.name)
            }
            Expr::Get(get_expr) => {
                format!("(get {})", get_expr.name)
            }
            Expr::Call(call_expr) => {
                let CallExpr {
                    callee,
                    paren: _,
                    args,
                } = call_expr;

                // Print callee concisely: if it's a simple variable, use its lexeme;
                // otherwise use the expression's print but strip a leading "call "
                let callee_repr = match *callee {
                    Expr::Var(var_expr) => var_expr.name.lexeme.to_string(),
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

                AstPrinter::parenthesize(&operator.lexeme, vec![left, right])
            }
            Expr::Logical(logical) => {
                let LogicalExpr {
                    left,
                    operator,
                    right,
                } = logical;

                AstPrinter::parenthesize(&operator.lexeme, vec![left, right])
            }
            Expr::Grouping(group) => AstPrinter::parenthesize("group", vec![group.expression]),
            Expr::Literal(val) => match val {
                LiteralExpr::Nil => "nil".to_string(),
                LiteralExpr::Boolean(bool) => bool.to_string(),
                LiteralExpr::Number(num) => num.to_string(),
                LiteralExpr::String(str) => str.to_string(),
                LiteralExpr::Call(_) => "<callable>".to_string(),
                LiteralExpr::Instance(_) => "<instance>".to_string(),
            },
            Expr::Unary(unary) => {
                let UnaryExpr { operator, right } = unary;

                AstPrinter::parenthesize(&operator.lexeme, vec![right])
            }
            Expr::Var(var_expr) => {
                format!("var {}", var_expr.name.lexeme)
            }
            Expr::Assign(assign) => {
                format!("Assign {} to {}", assign.value.print(), assign.name.lexeme)
            }
        }
    }
}

fn pad(f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
    write!(f, "{:indent$}", "", indent = level * 2)
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl Stmt {
    pub fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => {
                pad(f, level)?;
                writeln!(f, "ExprStmt")?;
                expr.fmt_indented(f, level + 1)
            }
            Stmt::Print(expr) => {
                pad(f, level)?;
                writeln!(f, "Print")?;
                expr.fmt_indented(f, level + 1)
            }
            Stmt::Var(v) => {
                pad(f, level)?;
                writeln!(f, "Var {}", v.name.lexeme)?;
                v.val.fmt_indented(f, level + 1)
            }
            Stmt::If(s) => {
                pad(f, level)?;
                writeln!(f, "If")?;
                pad(f, level + 1)?;
                writeln!(f, "Condition:")?;
                s.condition.fmt_indented(f, level + 2)?;
                pad(f, level + 1)?;
                writeln!(f, "Then:")?;
                s.then_b.fmt_indented(f, level + 2)?;
                if let Stmt::Block(b) = &*s.else_b {
                    if b.is_empty() {
                        return Ok(());
                    }
                }
                pad(f, level + 1)?;
                writeln!(f, "Else:")?;
                s.else_b.fmt_indented(f, level + 2)
            }
            Stmt::While(s) => {
                pad(f, level)?;
                writeln!(f, "While")?;
                pad(f, level + 1)?;
                writeln!(f, "Condition:")?;
                s.condition.fmt_indented(f, level + 2)?;
                pad(f, level + 1)?;
                writeln!(f, "Body:")?;
                s.body.fmt_indented(f, level + 2)
            }
            Stmt::Function(f_stmt) => {
                pad(f, level)?;
                writeln!(f, "Fun {}", f_stmt.name.lexeme)?;
                pad(f, level + 1)?;
                writeln!(
                    f,
                    "Params: {:?}",
                    f_stmt.params.iter().map(|t| &t.lexeme).collect::<Vec<_>>()
                )?;
                f_stmt.body.fmt_indented(f, level + 1)
            }
            Stmt::Block(stmts) => {
                pad(f, level)?;
                writeln!(f, "Block")?;
                for stmt in stmts {
                    stmt.fmt_indented(f, level + 1)?;
                }
                Ok(())
            }
            Stmt::Return(r) => {
                pad(f, level)?;
                writeln!(f, "Return")?;
                r.value.fmt_indented(f, level + 1)
            }
            Stmt::Class(c) => {
                pad(f, level)?;
                writeln!(f, "Class {}", c.name.lexeme)?;
                for method in &c.methods {
                    pad(f, level + 1)?;
                    writeln!(f, "Method {}", method.name.lexeme)?;
                    method.body.fmt_indented(f, level + 2)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl Expr {
    pub fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        match self {
            Expr::Super(s) => {
                pad(f, level)?;
                writeln!(f, "Super {}", s.method)
            }
            Expr::Assign(a) => {
                pad(f, level)?;
                writeln!(f, "Assign {} (depth: {:?})", a.name.lexeme, a.depth)?;
                a.value.fmt_indented(f, level + 1)
            }
            Expr::Binary(b) => {
                pad(f, level)?;
                writeln!(f, "Binary {}", b.operator.lexeme)?;
                b.left.fmt_indented(f, level + 1)?;
                b.right.fmt_indented(f, level + 1)
            }
            Expr::Logical(l) => {
                pad(f, level)?;
                writeln!(f, "Logical {}", l.operator.lexeme)?;
                l.left.fmt_indented(f, level + 1)?;
                l.right.fmt_indented(f, level + 1)
            }
            Expr::Grouping(g) => {
                pad(f, level)?;
                writeln!(f, "Group")?;
                g.expression.fmt_indented(f, level + 1)
            }
            Expr::Literal(l) => {
                pad(f, level)?;
                write!(f, "Literal ")?;
                l.fmt_indented(f, level)
            }
            Expr::Unary(u) => {
                pad(f, level)?;
                writeln!(f, "Unary {}", u.operator.lexeme)?;
                u.right.fmt_indented(f, level + 1)
            }
            Expr::This(t) => {
                pad(f, level)?;
                writeln!(f, "This (depth: {:?})", t.depth)
            }
            Expr::Var(v) => {
                pad(f, level)?;
                writeln!(f, "Var {} (depth: {:?})", v.name.lexeme, v.depth)
            }
            Expr::Call(c) => {
                pad(f, level)?;
                writeln!(f, "Call")?;
                c.callee.fmt_indented(f, level + 1)?;
                for arg in &c.args {
                    arg.fmt_indented(f, level + 1)?;
                }
                Ok(())
            }
            Expr::Get(g) => {
                pad(f, level)?;
                writeln!(f, "Get {}", g.name.lexeme)?;
                g.object.fmt_indented(f, level + 1)
            }
            Expr::Set(s) => {
                pad(f, level)?;
                writeln!(f, "Set {}", s.name.lexeme)?;
                s.object.fmt_indented(f, level + 1)?;
                s.value.fmt_indented(f, level + 1)
            }
        }
    }
}

impl fmt::Display for LiteralExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralExpr::Nil => write!(f, "nil"),
            LiteralExpr::Boolean(b) => write!(f, "{}", b),
            LiteralExpr::Number(n) => write!(f, "{}", n),
            LiteralExpr::String(s) => write!(f, "\"{}\"", s),
            LiteralExpr::Call(_) => write!(f, "<callable>"),
            LiteralExpr::Instance(_) => write!(f, "<instance>"),
        }
    }
}

impl LiteralExpr {
    pub fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, _: usize) -> fmt::Result {
        match self {
            LiteralExpr::Call(_) => write!(f, "<callable>"),
            LiteralExpr::Instance(_) => write!(f, "<instance>"),
            _ => writeln!(f, "{}", self),
        }
    }
}

// endregion: Formatting & Display

// region: Other Traits

impl PartialEq for NativeFn {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity
    }
}

// endregion: Other Traits
