use std::collections::HashMap;

use crate::{
    errors::{Locate, LoxError, ParseError},
    lox::{
        ast::{
            AssignmentExpr, BinaryExpr, CallExpr, ClassStmt, Expr, FunStmt, GetExpr, GroupingExpr,
            IfStmt, LiteralExpr, LogicalExpr, ReturnStmt, SetExpr, Stmt, ThisExpr, UnaryExpr,
            VarExpr, VarStmt, WhileStmt,
        },
        interpreter::Interpreter,
        token::Token,
    },
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum FunctionType {
    Initializer,
    Function,
    Method,
    None,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ClassType {
    Class,
    None,
}

pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    function: FunctionType,
    class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            function: FunctionType::None,
            class: ClassType::None,
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
            Expr::Set(set) => self.rs_set_expr(set),
            Expr::This(this) => self.rs_this_expr(this),
            Expr::Literal(_) => Ok(()),
        }
    }

    fn rs_class_stmt(&mut self, class: &mut ClassStmt) -> Result<(), LoxError> {
        let enclosing_class = self.class;
        self.class = ClassType::Class;

        self.declare(&class.name)?;
        self.define(&class.name)?;

        self.begin_scope();

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert("this".to_string(), true);
        }

        for method in &mut class.methods {
            let mut fn_type = FunctionType::Method;

            if method.name.lexeme == String::from("init") {
                fn_type = FunctionType::Initializer;
            }

            self.rs_function(method, fn_type)?;
        }

        self.end_scope();

        self.class = enclosing_class;

        Ok(())
    }

    fn rs_fun_stmt(&mut self, fun: &mut FunStmt) -> Result<(), LoxError> {
        self.declare(&fun.name)?;
        self.define(&fun.name)?;

        self.rs_function(fun, FunctionType::Function)
    }

    fn rs_function(&mut self, fun: &mut FunStmt, type_: FunctionType) -> Result<(), LoxError> {
        let enclosing_fn = self.function;
        self.function = type_;

        self.begin_scope();
        for param in &fun.params {
            self.declare(param)?;
            self.define(param)?;
        }
        self.resolve(&mut fun.body)?;
        self.end_scope();

        self.function = enclosing_fn;

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
        if self.function == FunctionType::None {
            return Err(ParseError::TopLevelReturn.at(return_.keyword.line));
        }

        if return_.value != LiteralExpr::Nil.into() && self.function == FunctionType::Initializer {
            return Err(ParseError::ReturnInAnInitializer.at(return_.keyword.line));
        }

        self.rs_expression(&mut return_.value)
    }

    fn rs_while_stmt(&mut self, while_: &mut WhileStmt) -> Result<(), LoxError> {
        self.rs_expression(&mut while_.condition)?;

        self.resolve(&mut while_.body)
    }

    fn rs_this_expr(&mut self, this: &mut ThisExpr) -> Result<(), LoxError> {
        if let ClassType::None = self.class {
            return Err(ParseError::OutsideThis.at(this.keyword.line));
        }

        self.resolve_local("this", &mut this.depth)
    }

    fn rs_set_expr(&mut self, set: &mut SetExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut set.object)?;
        self.rs_expression(&mut set.value)
    }

    fn rs_get_expr(&mut self, get: &mut GetExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut get.object)
    }

    fn rs_var_expr(&mut self, var: &mut VarExpr) -> Result<(), LoxError> {
        if let Some(scope) = self.scopes.last() {
            if let Some(initialized) = scope.get(&var.name.lexeme) {
                if !*initialized {
                    return Err(ParseError::SelfReferencingInitializer.at(var.name.line));
                }
            }
        }

        self.resolve_local(&var.name.lexeme, &mut var.depth)?;
        Ok(())
    }

    fn rs_assign_expr(&mut self, assign: &mut AssignmentExpr) -> Result<(), LoxError> {
        self.rs_expression(&mut assign.value)?;
        self.resolve_local(&assign.name.lexeme, &mut assign.depth)?;

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

    fn resolve_local(&mut self, name: &str, depth: &mut Option<usize>) -> Result<(), LoxError> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
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

        if scope.contains_key(&name.lexeme) {
            return Err(ParseError::VariableAlreadyDefined.at(name.line));
        }

        scope.insert(name.lexeme.clone(), false);

        Ok(())
    }

    fn define(&mut self, name: &Token) -> Result<(), LoxError> {
        let Some(scope) = self.scopes.last_mut() else {
            return Ok(());
        };

        scope.insert(name.lexeme.clone(), true);

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
