use crate::*;


#[derive(Clone, Debug)]
pub enum Expr
{
	Literal(diagn::Span, Value),
	Variable(diagn::Span, usize, Vec<String>),
	UnaryOp(diagn::Span, diagn::Span, UnaryOp, Box<Expr>),
	BinaryOp(diagn::Span, diagn::Span, BinaryOp, Box<Expr>, Box<Expr>),
	TernaryOp(diagn::Span, Box<Expr>, Box<Expr>, Box<Expr>),
	BitSlice(diagn::Span, diagn::Span, usize, usize, Box<Expr>),
	SoftSlice(diagn::Span, diagn::Span, usize, usize, Box<Expr>),
	Block(diagn::Span, Vec<Expr>),
	Call(diagn::Span, Box<Expr>, Vec<Expr>),
	Asm(diagn::Span, Vec<syntax::Token>),
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value
{
	Void,
	Integer(util::BigInt),
	Bool(bool),
	Function(String)
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnaryOp
{
	Neg,
	Not
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinaryOp
{
	Assign,
	
	Add, Sub, Mul, Div, Mod,
	Shl, Shr,
	And, Or, Xor,
	
	Eq, Ne,
	Lt, Le,
	Gt, Ge,
	
	LazyAnd, LazyOr,
	
	Concat
}


impl Expr
{
	pub fn new_dummy() -> Expr
	{
		Expr::Literal(diagn::Span::new_dummy(), Value::Bool(false))
	}

	
	pub fn span(&self) -> diagn::Span
	{
		match self
		{
			&Expr::Literal  (ref span, ..) => span.clone(),
			&Expr::Variable (ref span, ..) => span.clone(),
			&Expr::UnaryOp  (ref span, ..) => span.clone(),
			&Expr::BinaryOp (ref span, ..) => span.clone(),
			&Expr::TernaryOp(ref span, ..) => span.clone(),
			&Expr::BitSlice (ref span, ..) => span.clone(),
			&Expr::SoftSlice(ref span, ..) => span.clone(),
			&Expr::Block    (ref span, ..) => span.clone(),
			&Expr::Call     (ref span, ..) => span.clone(),
			&Expr::Asm      (ref span, ..) => span.clone(),
		}
	}
}


impl Value
{
	pub fn make_literal(&self) -> Expr
	{
		Expr::Literal(diagn::Span::new_dummy(), self.clone())
	}


	pub fn make_integer<T: Into<util::BigInt>>(value: T) -> Value
	{
		Value::Integer(value.into())
	}


	pub fn get_bigint(&self) -> Option<util::BigInt>
	{
		match self
		{
			&Value::Integer(ref bigint) => Some(bigint.clone()),
			_ => None,
		}
	}
}