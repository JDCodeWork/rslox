use crate::lox::expr::Binary;
use crate::lox::expr::{Expr, Grouping, Literal, Unary};
use crate::lox::token::*;

pub struct Interpreter;

impl Interpreter {
    pub fn grouping_expr(group: Grouping) -> Result<Literal, ()> {
        Interpreter::evaluate(*group.expression)
    }

    pub fn binary_expr(binary: Binary) -> Result<Literal, ()> {
        let left_expr = Interpreter::evaluate(*binary.left)?;
        let right_expr = Interpreter::evaluate(*binary.right)?;

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
                _ => return Ok(Literal::Nil),
            }
        }

        let left_num = match left_expr {
            Literal::Number(num) => num,
            _ => 0.0,
        };

        let right_num = match right_expr {
            Literal::Number(num) => num,
            _ => 0.0,
        };

        match *binary.operator.get_type() {
            TokenType::Minus => Ok(Literal::Number(left_num - right_num)),
            TokenType::Slash => Ok(Literal::Number(left_num / right_num)),
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

    pub fn unary_expr(unary: Unary) -> Result<Literal, ()> {
        let right = Interpreter::evaluate(*unary.right)?;

        match (unary.operator.get_type(), right) {
            (TokenType::Minus, Literal::Number(num)) => Ok(Literal::Number(-num)),
            (TokenType::Bang, lit) => {
                let bool_val = Interpreter::is_truthy(lit)?;
                Ok(Literal::Boolean(!bool_val))
            }
            _ => Ok(Literal::Nil),
        }
    }

    pub fn literal_expr(lit: Literal) -> Result<Literal, ()> {
        Ok(lit)
    }

    pub fn is_truthy(lit: Literal) -> Result<bool, ()> {
        match lit {
            Literal::Boolean(value) => Ok(value),
            Literal::Number(value) => Ok(value != 0.0),
            Literal::String(ref value) => Ok(!value.is_empty()),
            Literal::Nil => Ok(false),
        }
    }

    pub fn is_equal(left_lit: Literal, right_lit: Literal) -> Result<bool, ()> {
        match (&left_lit, &right_lit) {
            (Literal::Nil, Literal::Nil) => Ok(true),
            (Literal::Nil, _) => Ok(false),
            (Literal::String(left_str), Literal::String(right_str)) => Ok(left_str == right_str),
            _ => Ok(left_lit == right_lit),
        }
    }

    pub fn evaluate(expr: Expr) -> Result<Literal, ()> {
        match expr {
            Expr::Binary(binary) => Interpreter::binary_expr(binary),
            Expr::Grouping(group) => Interpreter::grouping_expr(group),
            Expr::Literal(literal) => Interpreter::literal_expr(literal),
            Expr::Unary(unary) => Interpreter::unary_expr(unary),
        }
    }
}
