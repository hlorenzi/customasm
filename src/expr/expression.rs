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
	Unknown,
	Void,
	Integer(util::BigInt),
	String(ValueString),
	Bool(bool),
	Function(String)
}


#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum StringEncoding
{
	Utf8,
	Utf16BE,
	Utf16LE,
	UnicodeBE,
	UnicodeLE,
	Ascii,
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueString
{
	pub utf8_contents: String,
	pub encoding: StringEncoding,
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


	pub fn make_string(s: &str, encoding: expr::StringEncoding) -> Value
	{
		Value::String(ValueString {
			utf8_contents: s.to_string(),
			encoding
		})
	}


	pub fn get_bigint(&self) -> Option<util::BigInt>
	{
		match self
		{
			&Value::Unknown => Some(util::BigInt::from(0)),
			&Value::Integer(ref bigint) => Some(bigint.clone()),
			&Value::String(ref rc_string) => Some(rc_string.to_bigint()),
			_ => None,
		}
	}
}


impl ValueString
{
	pub fn to_bigint(&self) -> util::BigInt
	{
		let bytes: Vec<u8> = match self.encoding
		{
    		StringEncoding::Utf8 => self.utf8_contents.bytes().collect(),
    		StringEncoding::Utf16BE => self.utf8_contents.encode_utf16().flat_map(|v| v.to_be_bytes()).collect(),
			StringEncoding::Utf16LE => self.utf8_contents.encode_utf16().flat_map(|v| v.to_le_bytes()).collect(),
    		StringEncoding::UnicodeBE => self.utf8_contents.chars().flat_map(|c| (c as u32).to_be_bytes()).collect(),
			StringEncoding::UnicodeLE => self.utf8_contents.chars().flat_map(|c| (c as u32).to_le_bytes()).collect(),
			StringEncoding::Ascii => self.utf8_contents.chars().map(|c| (c as u32) as u8).collect(), // can potentially contain invalid chars
		};
		util::BigInt::from_bytes_be(&bytes)
	}
}