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
	BuiltInFunction(String),
	Function(usize),
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueString
{
	pub utf8_contents: String,
	pub encoding: String,
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


	pub fn make_string<T: Into<String>, S: Into<String>>(value: T, encoding: S) -> Value
	{
		Value::String(ValueString
		{
			utf8_contents: value.into(),
			encoding: encoding.into(),
		})
	}


	pub fn get_bigint(&self) -> Option<util::BigInt>
	{
		match self
		{
			&Value::Unknown => Some(util::BigInt::from(0)),
			&Value::Integer(ref bigint) => Some(bigint.clone()),
			&Value::String(ref s) => Some(s.to_bigint()),
			_ => None,
		}
	}
}


impl ValueString
{
	pub fn to_bigint(&self) -> util::BigInt
	{
		match &*self.encoding
		{
			"utf8" => util::BigInt::from_bytes_be(&self.utf8_contents.as_bytes()),
			"utf16be" =>
			{
				let units = self.utf8_contents.encode_utf16();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit >> 8) & 0xff) as u8);
					bytes.push(((unit >> 0) & 0xff) as u8);
				}
					
				util::BigInt::from_bytes_be(&bytes[..])
			}
			"utf16le" =>
			{
				let units = self.utf8_contents.encode_utf16();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit >> 0) & 0xff) as u8);
					bytes.push(((unit >> 8) & 0xff) as u8);
				}
					
				util::BigInt::from_bytes_be(&bytes[..])
			}
			"utf32be" =>
			{
				let units = self.utf8_contents.chars();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit as u32 >> 24) & 0xff) as u8);
					bytes.push(((unit as u32 >> 16) & 0xff) as u8);
					bytes.push(((unit as u32 >> 8) & 0xff) as u8);
					bytes.push(((unit as u32 >> 0) & 0xff) as u8);
				}
					
				util::BigInt::from_bytes_be(&bytes[..])
			}
			"utf32le" =>
			{
				let units = self.utf8_contents.chars();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit as u32 >> 0) & 0xff) as u8);
					bytes.push(((unit as u32 >> 8) & 0xff) as u8);
					bytes.push(((unit as u32 >> 16) & 0xff) as u8);
					bytes.push(((unit as u32 >> 24) & 0xff) as u8);
				}
					
				util::BigInt::from_bytes_be(&bytes[..])
			}
			"ascii" =>
			{
				let units = self.utf8_contents.chars();
				let bytes = units.map(|c|
				{
					if c as u32 >= 0x100
					{
						0x00
					}
					else
					{
						c as u8
					}
				});
					
				util::BigInt::from_bytes_be(&bytes.collect::<Vec<_>>()[..])
			}
			_ => panic!("invalid string encoding"),
		}
	}
}