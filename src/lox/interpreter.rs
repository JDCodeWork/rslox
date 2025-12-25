use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::{Err, RuntimeErr};
use crate::lox::ast::*;
use crate::lox::env::{EnvBindings, Environment};
use crate::lox::token::*;

#[derive(Debug)]
pub enum ExecResult {
    Normal,
    Return(LiteralExpr),
}

#[derive(Default, Debug)]
pub struct Interpreter {
    pub(crate) env: Environment,
}

fn clock(_: &mut Interpreter, _: Vec<LiteralExpr>) -> Result<LiteralExpr, Err> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    Ok(LiteralExpr::Number(time))
}

impl Interpreter {
    pub fn interpret(stmts: Vec<Stmt>) -> Result<(), Err> {
        let mut executer = Interpreter::default();

        executer
            .env
            .define(String::from("clock"), NativeFn::new(0, clock).into());

        for stmt in stmts {
            executer.execute(stmt)?;
        }

        Ok(())
    }

    fn return_statement(&mut self, return_stmt: ReturnStmt) -> Result<ExecResult, Err> {
        let mut val = LiteralExpr::Nil;

        if return_stmt.value != LiteralExpr::Nil.into() {
            val = self.evaluate(return_stmt.value)?;
        }

        Ok(ExecResult::Return(val))
    }

    fn fun_statement(&mut self, mut fun_stmt: FunStmt) -> Result<ExecResult, Err> {
        let fn_name = fun_stmt.name.get_lexeme();

        fun_stmt.closure = Some(self.env.curr_node);

        let fun: Callable = fun_stmt.into();
        self.env.define(fn_name, fun.into());

        Ok(ExecResult::Normal)
    }

    fn if_statement(&mut self, if_stmt: IfStmt) -> Result<ExecResult, Err> {
        let mut result = ExecResult::Normal;

        if Self::is_truthy(self.evaluate(if_stmt.condition)?)? {
            result = self.execute(*if_stmt.then_b)?;
        } else if *if_stmt.else_b != LiteralExpr::Nil.into() {
            result = self.execute(*if_stmt.else_b)?;
        }

        Ok(result)
    }

    fn var_statement(&mut self, var_stmt: VarStmt) -> Result<ExecResult, Err> {
        let value = self.evaluate(var_stmt.val)?;

        self.env.define(var_stmt.name.get_lexeme(), value);
        Ok(ExecResult::Normal)
    }

    fn while_statement(&mut self, while_stmt: WhileStmt) -> Result<ExecResult, Err> {
        let WhileStmt { condition, body } = while_stmt;

        while Self::is_truthy(self.evaluate(condition.clone())?)? {
            let result = self.execute(*body.clone())?;

            if let ExecResult::Return(_) = result {
                return Ok(result);
            }
        }

        Ok(ExecResult::Normal)
    }

    fn expr_statement(&mut self, expr: Expr) -> Result<ExecResult, Err> {
        self.evaluate(expr)?;

        Ok(ExecResult::Normal)
    }

    fn print_statement(&mut self, expr: Expr) -> Result<ExecResult, Err> {
        let val: Expr = self.evaluate(expr)?.into();
        println!("{}", val.print());

        Ok(ExecResult::Normal)
    }

    fn call_expr(&mut self, call: CallExpr) -> Result<LiteralExpr, Err> {
        let callee = self.evaluate(*call.callee)?;

        let mut arguments = Vec::new();
        for arg in call.args {
            arguments.push(self.evaluate(arg)?);
        }

        let LiteralExpr::Call(mut callable) = callee else {
            return Err(RuntimeErr::InvalidCalleeExpr.into());
        };

        if arguments.len() != callable.arity() {
            return Err(
                RuntimeErr::ArgumentCountMismatch(callable.arity(), arguments.len()).into(),
            );
        }

        let val = callable.call(self, arguments)?;

        Ok(val)
    }

    fn assign_expr(&mut self, assign: AssignmentExpr) -> Result<LiteralExpr, Err> {
        let val = self.evaluate(*assign.value)?;
        self.env.assign(assign.name, val.clone())?;

        Ok(val)
    }

    fn var_expr(&self, name: Token) -> Result<LiteralExpr, Err> {
        self.env.get(&name)
    }

    fn grouping_expr(&mut self, group: GroupingExpr) -> Result<LiteralExpr, Err> {
        self.evaluate(*group.expression)
    }

    fn binary_expr(&mut self, binary: BinaryExpr) -> Result<LiteralExpr, Err> {
        let left_expr = self.evaluate(*binary.left)?;
        let right_expr = self.evaluate(*binary.right)?;

        if *binary.operator.get_type() == TokenType::Plus {
            match (left_expr, right_expr) {
                (LiteralExpr::String(left_str), LiteralExpr::String(right_str)) => {
                    let str = format!("{left_str}{right_str}");
                    return Ok(LiteralExpr::String(str));
                }
                (LiteralExpr::String(left_str), LiteralExpr::Number(right_num)) => {
                    let str = format!("{left_str}{right_num}");
                    return Ok(LiteralExpr::String(str));
                }
                (LiteralExpr::Number(left_num), LiteralExpr::Number(right_num)) => {
                    return Ok(LiteralExpr::Number(left_num + right_num))
                }
                _ => return Err(RuntimeErr::InvalidOperandTypes.to_err()),
            }
        }

        let left_num = match left_expr {
            LiteralExpr::Number(num) => num,
            _ => return Err(Err::from(RuntimeErr::OperandMustBeNumber)),
        };

        let right_num = match right_expr {
            LiteralExpr::Number(num) => num,
            LiteralExpr::String(ref str) => str.len() as f64,
            _ => return Err(Err::from(RuntimeErr::OperandMustBeNumber)),
        };

        match *binary.operator.get_type() {
            TokenType::Minus => Ok(LiteralExpr::Number(left_num - right_num)),
            TokenType::Slash => {
                if right_num == 0.0 {
                    return Err(RuntimeErr::DivisionByZero.to_err());
                }
                Ok(LiteralExpr::Number(left_num / right_num))
            }
            TokenType::Star => Ok(LiteralExpr::Number(left_num * right_num)),

            TokenType::Greater => Ok(LiteralExpr::Boolean(left_num > right_num)),
            TokenType::GreaterEqual => Ok(LiteralExpr::Boolean(left_num >= right_num)),
            TokenType::Less => Ok(LiteralExpr::Boolean(left_num < right_num)),
            TokenType::LessEqual => Ok(LiteralExpr::Boolean(left_num <= right_num)),

            TokenType::BangEqual => Ok(LiteralExpr::Boolean(!Interpreter::is_equal(
                left_expr, right_expr,
            )?)),
            TokenType::EqualEqual => Ok(LiteralExpr::Boolean(Interpreter::is_equal(
                left_expr, right_expr,
            )?)),
            _ => Ok(LiteralExpr::Nil),
        }
    }

    fn logical_expr(&mut self, logical: LogicalExpr) -> Result<LiteralExpr, Err> {
        let left = self.evaluate(*logical.left)?;

        if *logical.operator.get_type() == TokenType::Or {
            if Self::is_truthy(left.clone())? {
                return Ok(left);
            }
        } else if !Self::is_truthy(left.clone())? {
            return Ok(left);
        }

        Ok(self.evaluate(*logical.right)?)
    }

    fn unary_expr(&mut self, unary: Unary) -> Result<LiteralExpr, Err> {
        let right = self.evaluate(*unary.right)?;

        match (unary.operator.get_type(), right) {
            (TokenType::Minus, LiteralExpr::Number(num)) => Ok(LiteralExpr::Number(-num)),
            (TokenType::Minus, _) => Err(Err::from(RuntimeErr::OperandMustBeNumber)),
            (TokenType::Bang, lit) => {
                let bool_val = Interpreter::is_truthy(lit)?;
                Ok(LiteralExpr::Boolean(!bool_val))
            }
            _ => Ok(LiteralExpr::Nil),
        }
    }

    fn literal_expr(lit: LiteralExpr) -> Result<LiteralExpr, Err> {
        Ok(lit)
    }

    fn is_truthy(lit: LiteralExpr) -> Result<bool, Err> {
        match lit {
            LiteralExpr::Boolean(value) => Ok(value),
            LiteralExpr::Number(value) => Ok(value != 0.0),
            LiteralExpr::String(ref value) => Ok(!value.is_empty()),
            LiteralExpr::Nil => Ok(false),
            LiteralExpr::Call(_) => Ok(true),
        }
    }

    fn is_equal(left_lit: LiteralExpr, right_lit: LiteralExpr) -> Result<bool, Err> {
        match (&left_lit, &right_lit) {
            (LiteralExpr::Nil, LiteralExpr::Nil) => Ok(true),
            (LiteralExpr::Nil, _) => Ok(false),
            (LiteralExpr::String(left_str), LiteralExpr::String(right_str)) => {
                Ok(left_str == right_str)
            }
            _ => Ok(left_lit == right_lit),
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<LiteralExpr, Err> {
        match expr {
            Expr::Binary(binary) => self.binary_expr(binary),
            Expr::Grouping(group) => self.grouping_expr(group),
            Expr::Literal(literal) => Self::literal_expr(literal),
            Expr::Unary(unary) => self.unary_expr(unary),
            Expr::Var(name) => self.var_expr(name),
            Expr::Assign(assign) => self.assign_expr(assign),
            Expr::Logical(logical) => self.logical_expr(logical),
            Expr::Call(call) => self.call_expr(call),
        }
    }

    fn execute_block(&mut self, stmts: Vec<Stmt>, kind: BlockKind) -> Result<ExecResult, Err> {
        if let BlockKind::Default = kind {
            self.env.push_node();
        }

        for stmt in stmts {
            let result = match self.execute(stmt) {
                Ok(res) => res,
                Err(some) => some.report_and_exit(1),
            };

            if let ExecResult::Return(_) = result {
                self.env.pop_node();
                return Ok(result);
            }
        }

        self.env.pop_node();

        Ok(ExecResult::Normal)
    }

    pub fn execute(&mut self, stmt: Stmt) -> Result<ExecResult, Err> {
        match stmt {
            Stmt::Expression(expr) => self.expr_statement(expr),
            Stmt::Print(val) => self.print_statement(val),
            Stmt::Var(var_stmt) => self.var_statement(var_stmt),
            Stmt::Block(stmts) => self.execute_block(stmts, BlockKind::Default),
            Stmt::If(if_stmt) => self.if_statement(if_stmt),
            Stmt::While(while_stmt) => self.while_statement(while_stmt),
            Stmt::Function(fn_) => self.fun_statement(fn_),
            Stmt::Return(return_stmt) => self.return_statement(return_stmt),
        }
    }
}

impl Callable {
    pub fn arity(&self) -> usize {
        match self {
            Callable::User(fn_) => fn_.arity(),
            Callable::Native(fn_) => fn_.arity as usize,
        }
    }

    pub fn call(
        &mut self,
        exec: &mut Interpreter,
        args: Vec<LiteralExpr>,
    ) -> Result<LiteralExpr, Err> {
        match self {
            Callable::User(fn_) => fn_.call(exec, args),
            Callable::Native(fn_) => (fn_.action)(exec, args),
        }
    }
}

impl FunStmt {
    pub fn call(
        &mut self,
        exec: &mut Interpreter,
        args: Vec<LiteralExpr>,
    ) -> Result<LiteralExpr, Err> {
        let mut fun_bindings: EnvBindings = HashMap::new();

        for (param, value) in self.params.iter().zip(args) {
            fun_bindings.insert(param.get_lexeme(), value);
        }

        let stmts = match *self.body.clone() {
            Stmt::Block(stmts) => stmts,
            stmt => vec![stmt],
        };

        // To ensure the correct program execution we need the node when the function is called, because env.pop_node() only restores the environment to the state when the function was declared
        let previous = exec.env.curr_node;

        if let Some(closure) = self.closure {
            exec.env.push_closure(fun_bindings, closure);
        }

        let result = exec.execute_block(stmts, BlockKind::Closure);

        exec.env.curr_node = previous;

        let ExecResult::Return(val) = result? else {
            return Ok(LiteralExpr::Nil);
        };

        Ok(val)
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Err;
    use crate::lox::parser::Parser;
    use crate::lox::scanner::Scanner;

    fn eval_expr(src: &str) -> Result<LiteralExpr, Err> {
        let mut scanner = Scanner::new(src.to_string());
        let tokens = scanner.scan_tokens().clone();
        let mut parser = Parser::new(tokens);

        let stmts = parser.parse().map_err(Err::from)?;

        if let Some(stmt) = stmts.first() {
            match stmt {
                Stmt::Expression(expr) => {
                    let mut interpreter = Interpreter::default();
                    interpreter.evaluate(expr.clone())
                }
                Stmt::Print(expr) => {
                    let mut interpreter = Interpreter::default();
                    interpreter.evaluate(expr.clone())
                }
                _ => Err(Err::from(RuntimeErr::InvalidOperandTypes)),
            }
        } else {
            Err(Err::from(RuntimeErr::InvalidOperandTypes))
        }
    }

    fn run_src(src: &str) -> Result<(), Err> {
        let mut scanner = Scanner::new(src.to_string());
        let tokens = scanner.scan_tokens().clone();
        let mut parser = Parser::new(tokens);

        let stmts = parser.parse().map_err(Err::from)?;
        Interpreter::interpret(stmts)
    }

    fn parse_stmts(src: &str) -> Vec<Stmt> {
        let mut scanner = Scanner::new(src.to_string());
        let tokens = scanner.scan_tokens().clone();
        let mut parser = Parser::new(tokens);
        parser.parse().expect("Failed to parse statements")
    }

    #[test]
    fn test_interpreter_arithmetic() {
        let res = eval_expr("1 + 2 * 3;").expect("evaluation failed");
        assert_eq!(res, LiteralExpr::Number(7.0));
    }

    #[test]
    fn test_division_by_zero_returns_runtime_error() {
        let res = eval_expr("10 / 0;");
        assert!(res.is_err(), "Expected an error for division by zero");
        let err = res.unwrap_err();
        let dbg = format!("{:?}", err);
        // The Debug output should include the runtime error variant or message
        assert!(
            dbg.contains("Division") || dbg.contains("DivisionByZero") || dbg.contains("RUNTIME")
        );
    }

    #[test]
    fn test_unary_operand_type_error() {
        let res = eval_expr("-\"hello\";");
        assert!(res.is_err(), "Expected an error for negating a string");
        let err = res.unwrap_err();
        let dbg = format!("{:?}", err);
        assert!(
            dbg.contains("Operand")
                || dbg.contains("OperandMustBeNumber")
                || dbg.contains("RUNTIME")
        );
    }

    #[test]
    fn test_print_statement() {
        let res = run_src("print \"Hello, World! from tests\";");
        assert!(res.is_ok(), "Print statement should execute successfully");
    }

    #[test]
    fn test_expression_statement() {
        let res = run_src("1 + 2;");
        assert!(
            res.is_ok(),
            "Expression statement should execute successfully"
        );
    }

    #[test]
    fn test_variable_declaration() {
        let mut interpreter = Interpreter::default();
        let stmts = parse_stmts("var a = 1;");
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        // Check if 'a' is in the environment
        let token = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val = interpreter.env.get(&token).expect("variable lookup failed");
        assert_eq!(val, LiteralExpr::Number(1.0));
    }

    #[test]
    fn test_variable_assignment() {
        let mut interpreter = Interpreter::default();
        let stmts = parse_stmts("var a = 1; a = 2;");
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val = interpreter.env.get(&token).expect("variable lookup failed");
        assert_eq!(val, LiteralExpr::Number(2.0));
    }

    #[test]
    fn test_block_scope() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = \"global\";
            {
                var a = \"block\";
                var b = \"block_b\";
            }
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::String("global".to_string()));

        let token_b = Token::new(TokenType::Identifier, "b".to_string(), 1);
        let val_b = interpreter.env.get(&token_b);
        assert!(
            val_b.is_err(),
            "Variable b should not be accessible outside the block"
        );
    }

    #[test]
    fn test_scope_shadowing_and_assignment() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = 1;
            {
                var a = 2;
                a = 3;
            }
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::Number(1.0));
    }

    #[test]
    fn test_if_statement_true() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = 1;
            if (true) {
                a = 2;
            }
        ";

        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::Number(2.0));
    }

    #[test]
    fn test_if_statement_false() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = 1;
            if (false) {
                a = 2;
            }
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::Number(1.0));
    }

    #[test]
    fn test_if_else_statement_true() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = 1;
            if (true) {
                a = 2;
            } else {
                a = 3;
            }
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::Number(2.0));
    }

    #[test]
    fn test_if_else_statement_false() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = 1;
            if (false) {
                a = 2;
            } else {
                a = 3;
            }
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::Number(3.0));
    }

    #[test]
    fn test_logical_or() {
        let result = eval_expr("print 2 or \"hi\";").expect("evaluation failed");
        assert_eq!(result, LiteralExpr::Number(2.0));

        let result = eval_expr("print nil or \"hi\";").expect("evaluation failed");
        assert_eq!(result, LiteralExpr::String("hi".to_string()));
    }

    #[test]
    fn test_logical_and() {
        let result = eval_expr("print \"hi\" and 2;").expect("evaluation failed");
        assert_eq!(result, LiteralExpr::Number(2.0));

        let result = eval_expr("print nil and \"hi\";").expect("evaluation failed");
        assert_eq!(result, LiteralExpr::Nil);
    }

    #[test]
    fn test_while_statement() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = 1;
            while (a < 3) {
                a = a + 1;
            }
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::Number(3.0));
    }

    #[test]
    fn test_for_loop() {
        let mut interpreter = Interpreter::default();
        let src = "
            var a = 0;
            for (var i = 0; i < 3; i = i + 1) {
                a = a + i;
            }
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_a = Token::new(TokenType::Identifier, "a".to_string(), 1);
        let val_a = interpreter
            .env
            .get(&token_a)
            .expect("variable lookup failed");
        assert_eq!(val_a, LiteralExpr::Number(3.0));
    }

    #[test]
    fn test_function_declaration_and_call() {
        let mut interpreter = Interpreter::default();
        let src = "
            fun add(a, b) {
                return a + b;
            }
            var res = add(1, 2);
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_res = Token::new(TokenType::Identifier, "res".to_string(), 1);
        let val_res = interpreter
            .env
            .get(&token_res)
            .expect("variable lookup failed");
        assert_eq!(val_res, LiteralExpr::Number(3.0));
    }

    #[test]
    fn test_recursion_fibonacci() {
        let mut interpreter = Interpreter::default();
        let src = "
            fun fib(n) {
                if (n <= 1) return n;
                return fib(n - 2) + fib(n - 1);
            }
            var res = fib(10);
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_res = Token::new(TokenType::Identifier, "res".to_string(), 1);
        let val_res = interpreter
            .env
            .get(&token_res)
            .expect("variable lookup failed");
        assert_eq!(val_res, LiteralExpr::Number(55.0));
    }

    #[test]
    fn test_closure() {
        let mut interpreter = Interpreter::default();
        let src = "
            fun makeCounter() {
                var i = 0;
                fun count() {
                    i = i + 1;
                    return i;
                }
                return count;
            }

            var counter = makeCounter();
            var c1 = counter();
            var c2 = counter();
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_c1 = Token::new(TokenType::Identifier, "c1".to_string(), 1);
        let val_c1 = interpreter
            .env
            .get(&token_c1)
            .expect("variable lookup failed");
        assert_eq!(val_c1, LiteralExpr::Number(1.0));

        let token_c2 = Token::new(TokenType::Identifier, "c2".to_string(), 1);
        let val_c2 = interpreter
            .env
            .get(&token_c2)
            .expect("variable lookup failed");
        assert_eq!(val_c2, LiteralExpr::Number(2.0));
    }

    #[test]
    fn test_native_function_clock() {
        let mut interpreter = Interpreter::default();
        interpreter
            .env
            .define(String::from("clock"), NativeFn::new(0, clock).into());

        let src = "
            var t = clock();
        ";
        let stmts = parse_stmts(src);
        for stmt in stmts {
            interpreter.execute(stmt).expect("execution failed");
        }

        let token_t = Token::new(TokenType::Identifier, "t".to_string(), 1);
        let val_t = interpreter
            .env
            .get(&token_t)
            .expect("variable lookup failed");

        if let LiteralExpr::Number(_) = val_t {
            // Pass
        } else {
            panic!("clock() should return a number");
        }
    }
}
