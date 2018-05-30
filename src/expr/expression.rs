use diagn::Span;
use num_bigint::BigInt;


#[derive(Debug)]
pub enum Expression
{
	Literal(Span, ExpressionValue),
	Variable(Span, String),
	UnaryOp(Span, Span, UnaryOp, Box<Expression>),
	BinaryOp(Span, Span, BinaryOp, Box<Expression>, Box<Expression>),
	TernaryOp(Span, Box<Expression>, Box<Expression>, Box<Expression>),
	BitSlice(Span, Span, usize, usize, Box<Expression>),
	Block(Span, Vec<Expression>),
	Call(Span, Box<Expression>, Vec<Expression>)
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionValue
{
	Void,
	Integer(BigInt),
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
	pub fn span(&self) -> Span
	{
		match self
		{
			&Expression::Literal  (ref span, ..) => span.clone(),
			&Expression::Variable (ref span, ..) => span.clone(),
			&Expression::UnaryOp  (ref span, ..) => span.clone(),
			&Expression::BinaryOp (ref span, ..) => span.clone(),
			&Expression::TernaryOp(ref span, ..) => span.clone(),
			&Expression::BitSlice (ref span, ..) => span.clone(),
			&Expression::Block    (ref span, ..) => span.clone(),
			&Expression::Call     (ref span, ..) => span.clone()
		}
	}
}


impl ExpressionValue
{
	pub fn make_literal(&self) -> Expression
	{
		Expression::Literal(Span::new_dummy(), self.clone())
	}
}