use super::token::Token;

pub enum Expr {
	Binary(Binary),
	Grouping(Grouping),
	Literal(Literal),
	Unary(Unary),
}

pub struct Binary {
	left: Box<Expr>,
	operator: Token,
	right: Box<Expr>,
}

pub struct Grouping {
	expression: Box<Expr>,
}

pub struct Literal {
	value: String,
}

pub struct Unary {
	operator: Token,
	right: Box<Expr>,
}

impl Binary {
	pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
		Self {
			left: Box::new(left),
			operator,
			right: Box::new(right),
		}
	}
}

impl Grouping {
	pub fn new(expression: Expr) -> Self {
		Self {
			expression: Box::new(expression),
		}
	}
}

impl Literal {
	pub fn new(value: String) -> Self {
		Self {
			value,
		}
	}
}

impl Unary {
	pub fn new(operator: Token, right: Expr) -> Self {
		Self {
			operator,
			right: Box::new(right),
		}
	}
}

