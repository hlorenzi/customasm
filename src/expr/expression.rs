use crate::*;


#[derive(Debug)]
pub enum Expression
{
	Literal(diagn::Span, ExpressionValue),
	Variable(diagn::Span, String),
	UnaryOp(diagn::Span, diagn::Span, UnaryOp, Box<Expression>),
	BinaryOp(diagn::Span, diagn::Span, BinaryOp, Box<Expression>, Box<Expression>),
	TernaryOp(diagn::Span, Box<Expression>, Box<Expression>, Box<Expression>),
	BitSlice(diagn::Span, diagn::Span, usize, usize, Box<Expression>),
	SoftSlice(diagn::Span, diagn::Span, usize, usize, Box<Expression>),
	Block(diagn::Span, Vec<Expression>),
	Call(diagn::Span, Box<Expression>, Vec<Expression>)
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionValue
{
	Void,
	Integer(util::BigInt),
	Bool(bool),
	String(String),
	Function(usize)
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


impl Expression
{
	pub fn new_dummy() -> Expression
	{
		Expression::Literal(diagn::Span::new_dummy(), ExpressionValue::Bool(false))
	}

	
	pub fn span(&self) -> diagn::Span
	{
		match self
		{
			&Expression::Literal  (ref span, ..) => span.clone(),
			&Expression::Variable (ref span, ..) => span.clone(),
			&Expression::UnaryOp  (ref span, ..) => span.clone(),
			&Expression::BinaryOp (ref span, ..) => span.clone(),
			&Expression::TernaryOp(ref span, ..) => span.clone(),
			&Expression::BitSlice (ref span, ..) => span.clone(),
			&Expression::SoftSlice(ref span, ..) => span.clone(),
			&Expression::Block    (ref span, ..) => span.clone(),
			&Expression::Call     (ref span, ..) => span.clone()
		}
	}
}


impl ExpressionValue
{
	pub fn make_literal(&self) -> Expression
	{
		Expression::Literal(diagn::Span::new_dummy(), self.clone())
	}


	pub fn make_integer<T: Into<util::BigInt>>(value: T) -> ExpressionValue
	{
		ExpressionValue::Integer(value.into())
	}


	pub fn make_integer_from_usize(value: usize) -> ExpressionValue
	{
		ExpressionValue::Integer(value.into())
	}
}