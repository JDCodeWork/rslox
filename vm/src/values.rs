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
    Mod,
}

pub enum CompareOp {
    Equal,
    Greater,
    Less,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Constant {
    Number(f64),
    Boolean(bool),
    String { start: usize, end: usize },
    Nil,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Object(ObjRef),
    Nil,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ObjRef(pub usize);

pub enum Object {
    String(StrObj),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String(str) => write!(f, "{}", str.chars),
        }
    }
}

pub struct StrObj {
    pub lenght: usize,
    pub chars: Box<str>,
}

impl StrObj {
    pub fn new(s: &str) -> Self {
        Self {
            lenght: s.len(),
            chars: s.into(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Object(_) => write!(f, "obj"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(b) => !b,
            Value::Nil => true,
            Self::Object(_) => false,
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
                ArithOp::Mod => Ok(Value::Number(a % b)),
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
            (Value::Object(id_a), Value::Object(id_b)) => id_a == id_b,
            _ => false,
        }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::String { start: _s, end: _e } => write!(f, "str"),
            Self::Nil => write!(f, "nil"),
        }
    }
}
