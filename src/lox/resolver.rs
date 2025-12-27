use std::collections::HashMap;

use crate::{
    errors::{Err, ParseErr},
    lox::{
        ast::{
            AssignmentExpr, BinaryExpr, CallExpr, Expr, FunStmt, GroupingExpr, IfStmt, LiteralExpr,
            LogicalExpr, ReturnStmt, Stmt, UnaryExpr, VarStmt, WhileStmt,
        },
        interpreter::Interpreter,
        token::Token,
    },
};

pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    in_function: bool,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            in_function: false,
        }
    }

    fn resolve(&mut self, stmt: Stmt) -> Result<(), Err> {
        match stmt {
            Stmt::Var(var) => self.rs_var_stmt(var),
            Stmt::Expression(expr) => self.rs_expression(expr),
            Stmt::Function(fun) => self.rs_fun_stmt(fun),
            Stmt::If(if_) => self.rs_if_stmt(if_),
            Stmt::Block(stmts) => self.rs_block_stmt(stmts),
            Stmt::Print(value) => self.rs_print_stmt(value),
            Stmt::Return(value) => self.rs_return_stmt(value),
            Stmt::While(while_) => self.rs_while_stmt(while_),
        }
    }

    pub fn resolve_stmts(&mut self, stmts: Vec<Stmt>) -> Result<(), Err> {
        for stmt in stmts {
            self.resolve(stmt)?;
        }

        Ok(())
    }

    fn rs_expression(&mut self, expr: Expr) -> Result<(), Err> {
        match expr {
            Expr::Assign(assign) => self.rs_assign_expr(assign),
            Expr::Var(var) => self.rs_var_expr(var),
            Expr::Grouping(group) => self.rs_group_expr(group),
            Expr::Binary(bin) => self.rs_binary_expr(bin),
            Expr::Call(call) => self.rs_call_expr(call),
            Expr::Logical(logic) => self.rs_logic_expr(logic),
            Expr::Unary(unary) => self.rs_unary_expr(unary),
            Expr::Literal(_) => Ok(()),
        }
    }

    fn rs_fun_stmt(&mut self, fun: FunStmt) -> Result<(), Err> {
        self.declare(&fun.name)?;
        self.define(&fun.name)?;

        self.rs_function(fun)
    }

    fn rs_function(&mut self, fun: FunStmt) -> Result<(), Err> {
        let enclosing_fn = self.in_function;
        self.in_function = true;

        self.begin_scope();
        for param in fun.params {
            self.declare(&param)?;
            self.define(&param)?;
        }
        self.resolve(*fun.body)?;
        self.end_scope();

        self.in_function = enclosing_fn;

        Ok(())
    }

    fn rs_var_stmt(&mut self, var: VarStmt) -> Result<(), Err> {
        self.declare(&var.name)?;
        if var.val != LiteralExpr::Nil.into() {
            self.rs_expression(var.val)?;
        }
        self.define(&var.name)
    }

    fn rs_if_stmt(&mut self, if_: IfStmt) -> Result<(), Err> {
        self.rs_expression(if_.condition)?;

        self.resolve(*if_.else_b)?;
        self.resolve(*if_.then_b)
    }

    fn rs_print_stmt(&mut self, value: Expr) -> Result<(), Err> {
        self.rs_expression(value)
    }

    fn rs_return_stmt(&mut self, return_: ReturnStmt) -> Result<(), Err> {
        if !self.in_function {
            ParseErr::TopLevelReturn(return_.keyword.get_line())
                .into_err()
                .report_and_exit(1);
        }

        self.rs_expression(return_.value)
    }

    fn rs_while_stmt(&mut self, while_: WhileStmt) -> Result<(), Err> {
        self.rs_expression(while_.condition)?;

        self.resolve(*while_.body)
    }

    fn rs_var_expr(&mut self, var: Token) -> Result<(), Err> {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(ParseErr::InvalidLocalVariable(var.get_line()).into_err());
        };

        let Some(initialized) = scope.get(&var.get_lexeme()) else {
            return Err(ParseErr::InvalidLocalVariable(var.get_line()).into_err());
        };

        if !*initialized {
            return Err(ParseErr::InvalidLocalVariable(var.get_line()).into_err());
        }

        self.resolve_local(&var)
    }

    fn rs_assign_expr(&mut self, assign: AssignmentExpr) -> Result<(), Err> {
        self.rs_expression(*assign.value)?;
        self.resolve_local(&assign.name)?;

        Ok(())
    }

    fn rs_block_stmt(&mut self, stmts: Vec<Stmt>) -> Result<(), Err> {
        self.begin_scope();
        for stmt in stmts {
            self.resolve(stmt)?;
        }
        self.end_scope();

        Ok(())
    }

    fn rs_binary_expr(&mut self, bin: BinaryExpr) -> Result<(), Err> {
        self.rs_expression(*bin.left)?;
        self.rs_expression(*bin.right)
    }

    fn rs_call_expr(&mut self, call: CallExpr) -> Result<(), Err> {
        self.rs_expression(*call.callee)?;

        for arg in call.args {
            self.rs_expression(arg)?;
        }

        Ok(())
    }

    fn rs_group_expr(&mut self, group: GroupingExpr) -> Result<(), Err> {
        self.rs_expression(*group.expression)
    }

    fn rs_logic_expr(&mut self, logic: LogicalExpr) -> Result<(), Err> {
        self.rs_expression(*logic.left)?;
        self.rs_expression(*logic.right)
    }

    fn rs_unary_expr(&mut self, unary: UnaryExpr) -> Result<(), Err> {
        self.rs_expression(*unary.right)
    }

    fn resolve_local(&mut self, name: &Token) -> Result<(), Err> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.get_lexeme()) {
                self.interpreter.resolve(name, self.scopes.len() - 1 - i);
                return Ok(());
            }
        }

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), Err> {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(
                ParseErr::ExpectedToken("Expected block".to_string(), name.get_line()).into_err(),
            );
        };

        if scope.contains_key(&name.get_lexeme()) {
            return Err(ParseErr::VariablesWithSameName(name.get_line()).into_err());
        }

        scope.insert(name.get_lexeme(), false);

        Ok(())
    }

    fn define(&mut self, name: &Token) -> Result<(), Err> {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(
                ParseErr::ExpectedToken("Expected block".to_string(), name.get_line()).into_err(),
            );
        };

        scope.insert(name.get_lexeme(), true);

        Ok(())
    }
}
