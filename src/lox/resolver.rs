use std::collections::HashMap;

use crate::{
    errors::{Locate, LoxError, ParseError},
    lox::{
        ast::{
            AssignmentExpr, BinaryExpr, CallExpr, ClassStmt, Expr, FunStmt, GetExpr, GroupingExpr,
            IfStmt, LiteralExpr, LogicalExpr, ReturnStmt, Stmt, UnaryExpr, VarExpr, VarStmt,
            WhileStmt,
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

    fn resolve(&mut self, stmt: &mut Stmt) -> Result<(), LoxError> {
        match stmt {
            Stmt::Var(var) => self.rs_var_stmt(var),
            Stmt::Expression(expr) => self.rs_expression(expr),
            Stmt::Function(fun) => self.rs_fun_stmt(fun),
            Stmt::If(if_) => self.rs_if_stmt(if_),
            Stmt::Block(stmts) => self.rs_block_stmt(stmts),
            Stmt::Print(value) => self.rs_print_stmt(value),
            Stmt::Return(value) => self.rs_return_stmt(value),
            Stmt::While(while_) => self.rs_while_stmt(while_),
            Stmt::Class(class) => self.rs_class_stmt(class),
        }
    }

    pub fn resolve_stmts(&mut self, stmts: &mut Vec<Stmt>) -> Result<(), LoxError> {
        for stmt in stmts {
            self.resolve(stmt)?;
        }

        Ok(())
    }

    fn rs_expression(&mut self, expr: &mut Expr) -> Result<(), LoxError> {
        match expr {
            Expr::Assign(assign) => self.rs_assign_expr(assign),
            Expr::Var(var) => self.rs_var_expr(var),
            Expr::Grouping(group) => self.rs_group_expr(group),
            Expr::Binary(bin) => self.rs_binary_expr(bin),
            Expr::Call(call) => self.rs_call_expr(call),
            Expr::Logical(logic) => self.rs_logic_expr(logic),
            Expr::Unary(unary) => self.rs_unary_expr(unary),
            Expr::Get(get) => self.rs_get_expr(get),
            Expr::Literal(_) => Ok(()),
        }
    }

    fn rs_class_stmt(&mut self, class: &mut ClassStmt) -> Result<(), LoxError> {
        self.declare(&class.name)?;
        self.define(&class.name)
    }

    fn rs_fun_stmt(&mut self, fun: &mut FunStmt) -> Result<(), LoxError> {
        self.declare(&fun.name)?;
        self.define(&fun.name)?;

        self.rs_function(fun)
    }

    fn rs_function(&mut self, fun: &mut FunStmt) -> Result<(), LoxError> {
        let enclosing_fn = self.in_function;
        self.in_function = true;

        self.begin_scope();
        for param in &fun.params {
            self.declare(param)?;
            self.define(param)?;
        }
        self.resolve(&mut fun.body)?;
        self.end_scope();

        self.in_function = enclosing_fn;

        Ok(())
    }

    fn rs_var_stmt(&mut self, var: &mut VarStmt) -> Result<(), LoxError> {
        self.declare(&var.name)?;
        if var.val != LiteralExpr::Nil.into() {
            self.rs_expression(&mut var.val)?;
        }
        self.define(&var.name)
    }

    fn rs_if_stmt(&mut self, if_: &mut IfStmt) -> Result<(), LoxError> {
        self.rs_expression(&mut if_.condition)?;

        self.resolve(&mut if_.else_b)?;
        self.resolve(&mut if_.then_b)
    }

    fn rs_print_stmt(&mut self, value: &mut Expr) -> Result<(), LoxError> {
        self.rs_expression(value)
    }

    fn rs_return_stmt(&mut self, return_: &mut ReturnStmt) -> Result<(), LoxError> {
        if !self.in_function {
            return Err(ParseError::TopLevelReturn.at(return_.keyword.get_line()));
        }

        self.rs_expression(&mut return_.value)
    }

    fn rs_while_stmt(&mut self, while_: &mut WhileStmt) -> Result<(), LoxError> {
        self.rs_expression(&mut while_.condition)?;

        self.resolve(&mut while_.body)
    }

    fn rs_get_expr(&mut self, get: &mut GetExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut get.object)
    }

    fn rs_var_expr(&mut self, var: &mut VarExpr) -> Result<(), LoxError> {
        if let Some(scope) = self.scopes.last() {
            if let Some(initialized) = scope.get(&var.name.get_lexeme()) {
                if !*initialized {
                    return Err(ParseError::SelfReferencingInitializer.at(var.name.get_line()));
                }
            }
        }

        self.resolve_local(&var.name, &mut var.depth)?;
        Ok(())
    }

    fn rs_assign_expr(&mut self, assign: &mut AssignmentExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut assign.value)?;
        self.resolve_local(&assign.name, &mut assign.depth)?;

        Ok(())
    }

    fn rs_block_stmt(&mut self, stmts: &mut Vec<Stmt>) -> Result<(), LoxError> {
        self.begin_scope();
        for stmt in stmts {
            self.resolve(stmt)?;
        }
        self.end_scope();

        Ok(())
    }

    fn rs_binary_expr(&mut self, bin: &mut BinaryExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut bin.left)?;
        self.rs_expression(&mut bin.right)
    }

    fn rs_call_expr(&mut self, call: &mut CallExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut call.callee)?;

        for arg in &mut call.args {
            self.rs_expression(arg)?;
        }

        Ok(())
    }

    fn rs_group_expr(&mut self, group: &mut GroupingExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut group.expression)
    }

    fn rs_logic_expr(&mut self, logic: &mut LogicalExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut logic.left)?;
        self.rs_expression(&mut logic.right)
    }

    fn rs_unary_expr(&mut self, unary: &mut UnaryExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut unary.right)
    }

    fn resolve_local(&mut self, name: &Token, depth: &mut Option<usize>) -> Result<(), LoxError> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.get_lexeme()) {
                *depth = Some(i);

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

    fn declare(&mut self, name: &Token) -> Result<(), LoxError> {
        let Some(scope) = self.scopes.last_mut() else {
            return Ok(());
        };

        if scope.contains_key(&name.get_lexeme()) {
            return Err(ParseError::VariableAlreadyDefined.at(name.get_line()));
        }

        scope.insert(name.get_lexeme(), false);

        Ok(())
    }

    fn define(&mut self, name: &Token) -> Result<(), LoxError> {
        let Some(scope) = self.scopes.last_mut() else {
            return Ok(());
        };

        scope.insert(name.get_lexeme(), true);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lox::parser::Parser;
    use crate::lox::scanner::Scanner;

    fn resolve_src(src: &str) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(src.to_string());
        let tokens = scanner.scan_tokens().clone();
        let mut parser = Parser::new(tokens);

        let mut stmts = parser.parse()?;
        let mut resolver = Resolver::new(Interpreter::new());
        resolver.resolve_stmts(&mut stmts)
    }

    #[test]
    fn test_valid_resolution() {
        let src = "
            var a = 1;
            fun f() {
                print a;
            }
        ";
        assert!(resolve_src(src).is_ok());
    }

    #[test]
    fn test_var_redeclaration_error() {
        let src = "
            fun f() {
                var a = 1;
                var a = 2;
            }
        ";
        let res = resolve_src(src);
        assert!(res.is_err());
        let err = res.unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("VariableAlreadyDefined") || msg.contains("PARSE"));
    }

    #[test]
    fn test_top_level_return_error() {
        let src = "
            return 1;
        ";
        let res = resolve_src(src);
        assert!(res.is_err());
        let err = res.unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("TopLevelReturn") || msg.contains("PARSE"));
    }
}
