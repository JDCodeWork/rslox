use std::fmt;

pub enum ArithmeticError {
    DivisionByZero,
    InvalidOperands,
}

type ArithResult<T = Value> = Result<T, ArithmeticError>;

pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

pub enum CompareOp {
    Equal,
    Greater,
    Less,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(b) => !b,
            Value::Nil => true,
            Value::Number(_) => false,
        }
    }

    pub fn arithmetic(self, rhs: Self, op: ArithOp) -> ArithResult {
        if let (Value::Number(a), Value::Number(b)) = (self, rhs) {
            match op {
                ArithOp::Add => Ok(Value::Number(a + b)),
                ArithOp::Sub => Ok(Value::Number(a - b)),
                ArithOp::Mul => Ok(Value::Number(a * b)),
                ArithOp::Div => {
                    if b == 0.0 {
                        Err(ArithmeticError::DivisionByZero)
                    } else {
                        Ok(Value::Number(a / b))
                    }
                }
            }
        } else {
            Err(ArithmeticError::InvalidOperands)
        }
    }

    pub fn compare(self, rhs: Self, op: CompareOp) -> bool {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => match op {
                CompareOp::Equal => a == b,
                CompareOp::Greater => a > b,
                CompareOp::Less => a < b,
            },
            (Value::Boolean(a), Value::Boolean(b)) => match op {
                CompareOp::Equal => a == b,
                _ => false,
            },
            (Value::Nil, Value::Nil) => matches!(op, CompareOp::Equal),
            _ => false,
        }
    }
}
