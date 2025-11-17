use crate::errors::{Err, RuntimeErr};
use crate::lox::ast::{Assignment, Binary, Expr, Grouping, Literal, Stmt, Unary, VarStmt};
use crate::lox::env::Enviroment;
use crate::lox::token::*;

#[derive(Default)]
pub struct Interpreter {
    env: Enviroment,
}

impl Interpreter {
    pub fn interpret(stmts: Vec<Stmt>) -> Result<(), Err> {
        let mut ctx = Interpreter::default();

        for stmt in stmts {
            ctx.execute(stmt)?;
        }

        Ok(())
    }

    fn var_statement(&mut self, var_stmt: VarStmt) -> Result<(), Err> {
        let value = self.evaluate(var_stmt.val)?;

        self.env.define(var_stmt.name.get_lexeme(), value);
        Ok(())
    }

    fn expr_statement(&mut self, expr: Expr) -> Result<(), Err> {
        self.evaluate(expr)?;

        Ok(())
    }

    fn print_stament(&mut self, expr: Expr) -> Result<(), Err> {
        let val: Expr = self.evaluate(expr)?.into();
        println!("{}", val.print());

        Ok(())
    }

    fn assign_expr(&mut self, assign: Assignment) -> Result<Literal, Err> {
        let val = self.evaluate(*assign.value)?;
        self.env.assign(&assign.name.get_lexeme(), val.clone())?;

        Ok(val)
    }

    fn var_expr(&self, name: Token) -> Result<Literal, Err> {
        self.env.get(name.get_lexeme().as_str())
    }

    fn grouping_expr(&mut self, group: Grouping) -> Result<Literal, Err> {
        self.evaluate(*group.expression)
    }

    fn binary_expr(&mut self, binary: Binary) -> Result<Literal, Err> {
        let left_expr = self.evaluate(*binary.left)?;
        let right_expr = self.evaluate(*binary.right)?;

        if *binary.operator.get_type() == TokenType::Plus {
            match (left_expr, right_expr) {
                (Literal::String(left_str), Literal::String(right_str)) => {
                    let str = format!("{left_str}{right_str}");
                    return Ok(Literal::String(str));
                }
                (Literal::String(left_str), Literal::Number(right_num)) => {
                    let str = format!("{left_str}{right_num}");
                    return Ok(Literal::String(str));
                }
                (Literal::Number(left_num), Literal::Number(right_num)) => {
                    return Ok(Literal::Number(left_num + right_num))
                }
                _ => return Err(RuntimeErr::InvalidOperandTypes.to_err()),
            }
        }

        let left_num = match left_expr {
            Literal::Number(num) => num,
            _ => return Err(Err::from(RuntimeErr::OperandMustBeNumber)),
        };

        let right_num = match right_expr {
            Literal::Number(num) => num,
            Literal::String(ref str) => str.len() as f64,
            _ => return Err(Err::from(RuntimeErr::OperandMustBeNumber)),
        };

        match *binary.operator.get_type() {
            TokenType::Minus => Ok(Literal::Number(left_num - right_num)),
            TokenType::Slash => {
                if right_num == 0.0 {
                    return Err(RuntimeErr::DivisionByZero.to_err());
                }
                Ok(Literal::Number(left_num / right_num))
            }
            TokenType::Star => Ok(Literal::Number(left_num * right_num)),

            TokenType::Greater => Ok(Literal::Boolean(left_num > right_num)),
            TokenType::GreaterEqual => Ok(Literal::Boolean(left_num >= right_num)),
            TokenType::Less => Ok(Literal::Boolean(left_num < right_num)),
            TokenType::LessEqual => Ok(Literal::Boolean(left_num <= right_num)),

            TokenType::BangEqual => Ok(Literal::Boolean(!Interpreter::is_equal(
                left_expr, right_expr,
            )?)),
            TokenType::EqualEqual => Ok(Literal::Boolean(Interpreter::is_equal(
                left_expr, right_expr,
            )?)),
            _ => Ok(Literal::Nil),
        }
    }

    fn unary_expr(&mut self, unary: Unary) -> Result<Literal, Err> {
        let right = self.evaluate(*unary.right)?;

        match (unary.operator.get_type(), right) {
            (TokenType::Minus, Literal::Number(num)) => Ok(Literal::Number(-num)),
            (TokenType::Minus, _) => Err(Err::from(RuntimeErr::OperandMustBeNumber)),
            (TokenType::Bang, lit) => {
                let bool_val = Interpreter::is_truthy(lit)?;
                Ok(Literal::Boolean(!bool_val))
            }
            _ => Ok(Literal::Nil),
        }
    }

    fn literal_expr(lit: Literal) -> Result<Literal, Err> {
        Ok(lit)
    }

    fn is_truthy(lit: Literal) -> Result<bool, Err> {
        match lit {
            Literal::Boolean(value) => Ok(value),
            Literal::Number(value) => Ok(value != 0.0),
            Literal::String(ref value) => Ok(!value.is_empty()),
            Literal::Nil => Ok(false),
        }
    }

    fn is_equal(left_lit: Literal, right_lit: Literal) -> Result<bool, Err> {
        match (&left_lit, &right_lit) {
            (Literal::Nil, Literal::Nil) => Ok(true),
            (Literal::Nil, _) => Ok(false),
            (Literal::String(left_str), Literal::String(right_str)) => Ok(left_str == right_str),
            _ => Ok(left_lit == right_lit),
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Literal, Err> {
        match expr {
            Expr::Binary(binary) => self.binary_expr(binary),
            Expr::Grouping(group) => self.grouping_expr(group),
            Expr::Literal(literal) => Self::literal_expr(literal),
            Expr::Unary(unary) => self.unary_expr(unary),
            Expr::Var(name) => self.var_expr(name),
            Expr::Assign(assign) => self.assign_expr(assign),
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), Err> {
        match stmt {
            Stmt::Expression(expr) => self.expr_statement(expr),
            Stmt::Print(val) => self.print_stament(val),
            Stmt::Var(var_stmt) => self.var_statement(var_stmt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Err;
    use crate::lox::parser::Parser;
    use crate::lox::scanner::Scanner;

    fn eval_expr(src: &str) -> Result<Literal, Err> {
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

    #[test]
    fn test_interpreter_arithmetic() {
        let res = eval_expr("1 + 2 * 3;").expect("evaluation failed");
        assert_eq!(res, Literal::Number(7.0));
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
        let res = run_src("print \"Hello, World!\";");
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
}
