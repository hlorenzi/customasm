use crate::*;


#[derive(Clone, Debug)]
pub enum Expr
{
	Literal(diagn::Span, Value),
	Variable(diagn::Span, usize, Vec<String>),
	UnaryOp(diagn::Span, diagn::Span, UnaryOp, Box<Expr>),
	BinaryOp(diagn::Span, diagn::Span, BinaryOp, Box<Expr>, Box<Expr>),
	TernaryOp(diagn::Span, Box<Expr>, Box<Expr>, Box<Expr>),
	Slice(diagn::Span, diagn::Span, Box<Expr>, Box<Expr>, Box<Expr>),
	SliceShort(diagn::Span, diagn::Span, Box<Expr>, Box<Expr>),
	Block(diagn::Span, Vec<Expr>),
	Call(diagn::Span, Box<Expr>, Vec<Expr>),
	Asm(diagn::Span, asm::AstTopLevel),
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value
{
	Unknown,
	FailedConstraint(diagn::Message),
	Void,
	Integer(util::BigInt),
	String(ExprString),
	Bool(bool),
	ExprBuiltInFunction(String),
	AsmBuiltInFunction(String),
	Function(usize /*util::ItemRef<asm2::Function>*/),
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExprString
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
			&Expr::Literal   (span, ..) => span,
			&Expr::Variable  (span, ..) => span,
			&Expr::UnaryOp   (span, ..) => span,
			&Expr::BinaryOp  (span, ..) => span,
			&Expr::TernaryOp (span, ..) => span,
			&Expr::Slice     (span, ..) => span,
			&Expr::SliceShort(span, ..) => span,
			&Expr::Block     (span, ..) => span,
			&Expr::Call      (span, ..) => span,
			&Expr::Asm       (span, ..) => span,
		}
	}
}


impl Value
{
	pub fn is_unknown(&self) -> bool
	{
		match self
		{
			Value::Unknown => true,
			_ => false,
		}
	}


	pub fn should_propagate(&self) -> bool
	{
		match self
		{
			Value::Unknown => true,
			Value::FailedConstraint(_) => true,
			_ => false,
		}
	}

	
	pub fn make_literal(&self) -> Expr
	{
		Expr::Literal(diagn::Span::new_dummy(), self.clone())
	}


	pub fn make_integer<T: Into<util::BigInt>>(value: T) -> Value
	{
		Value::Integer(value.into())
	}


	pub fn make_bool(value: bool) -> Value
	{
		Value::Bool(value)
	}


	pub fn make_string<T: Into<String>, S: Into<String>>(value: T, encoding: S) -> Value
	{
		Value::String(ExprString
		{
			utf8_contents: value.into(),
			encoding: encoding.into(),
		})
	}


	pub fn get_bigint(&self) -> Option<util::BigInt>
	{
		match self
		{
			&Value::Integer(ref bigint) => Some(bigint.clone()),
			&Value::String(ref s) => Some(s.to_bigint()),
			_ => None,
		}
	}


	pub fn coallesce_to_integer<'a>(
		&'a self)
		-> std::borrow::Cow<'a, expr::Value>
	{
		match self
		{
			Value::String(ref s) =>
				std::borrow::Cow::Owned(
					expr::Value::Integer(s.to_bigint())),

			_ => std::borrow::Cow::Borrowed(self),
		}
	}


	pub fn unwrap_bigint(
		&self)
		-> &util::BigInt
	{
		match self
		{
			Value::Integer(bigint) => bigint,
			_ => panic!(),
		}
	}


	pub fn expect_bigint(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<&util::BigInt, ()>
	{
		match self
		{
			Value::Integer(bigint) => Ok(bigint),

			Value::Unknown =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					"expected integer",
					span);

				Err(())
			}
		}
	}


	pub fn expect_bigint_mut(
		&mut self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<&mut util::BigInt, ()>
	{
		match self
		{
			Value::Integer(ref mut bigint) => Ok(bigint),
			
			Value::Unknown =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					"expected integer",
					span);

				Err(())
			}
		}
	}


	pub fn expect_sized_bigint(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<&util::BigInt, ()>
	{
		let bigint = self.expect_bigint(report, span)?;
		
		match bigint.size
		{
			Some(_) => Ok(&bigint),
			
			None =>
			{
				report.error_span(
					"expected integer with definite size",
					span);

				Err(())
			}
		}
	}


	pub fn expect_sized_integerlike(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<(util::BigInt, usize), ()>
	{
		if let Some(bigint) = self.coallesce_to_integer().get_bigint()
		{
			if let Some(size) = bigint.size
			{
				return Ok((bigint, size));
			}

			report.error_span(
				"value has no definite size",
				span);
			
			return Err(());
		}

		report.error_span(
			"expected integer-like value with definite size",
			span);

		Err(())
	}


	pub fn expect_error_or_bigint(
		self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<expr::Value, ()>
	{
		match self.coallesce_to_integer().as_ref()
		{
			value @ expr::Value::Unknown |
			value @ expr::Value::FailedConstraint(_) =>
				Ok(value.to_owned()),

			value @ expr::Value::Integer(_) =>
				Ok(value.to_owned()),

			_ =>
			{
				report.error_span(
					"expected integer",
					span);

				Err(())
			}
		}
	}


	pub fn expect_error_or_sized_bigint(
		self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<expr::Value, ()>
	{
		match self.coallesce_to_integer().as_ref()
		{
			value @ expr::Value::Unknown |
			value @ expr::Value::FailedConstraint(_) =>
				Ok(value.to_owned()),

			expr::Value::Integer(bigint)
				if bigint.size.is_some() =>
				Ok(expr::Value::Integer(bigint.to_owned())),

			_ =>
			{
				report.error_span(
					"expected integer with definite size",
					span);

				Err(())
			}
		}
	}


	pub fn expect_error_or_bool(
		self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<expr::Value, ()>
	{
		match self.coallesce_to_integer().as_ref()
		{
			value @ expr::Value::Unknown |
			value @ expr::Value::FailedConstraint(_) =>
				Ok(value.to_owned()),

			value @ expr::Value::Bool(_) =>
				Ok(value.to_owned()),

			_ =>
			{
				report.error_span(
					"expected boolean",
					span);

				Err(())
			}
		}
	}


	pub fn as_usize(&self) -> Option<usize>
	{
		match self
		{
			Value::Integer(ref bigint) =>
				bigint.maybe_into::<usize>(),

			_ => None,
		}
	}


	pub fn expect_usize(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<usize, ()>
	{
		match self
		{
			Value::Integer(ref bigint) =>
				bigint.checked_into::<usize>(
					report,
					span),

			Value::Unknown =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					"expected non-negative integer",
					span);

				Err(())
			}
		}
	}


	pub fn expect_error_or_usize(
		self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<expr::Value, ()>
	{
		match &self
		{
			expr::Value::Unknown |
			expr::Value::FailedConstraint(_) =>
				Ok(self),

			Value::Integer(ref bigint) =>
			{
				bigint.checked_into::<usize>(
					report,
					span)?;

				Ok(self)
			}

			_ =>
			{
				report.error_span(
					"expected non-negative integer",
					span);

				Err(())
			}
		}
	}


	pub fn expect_nonzero_usize(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<usize, ()>
	{
		match self
		{
			Value::Integer(ref bigint) =>
			{
				bigint.checked_into_nonzero_usize(
					report,
					span)
			}

			Value::Unknown =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					"expected positive integer",
					span);

				Err(())
			}
		}
	}


	pub fn expect_bool(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<bool, ()>
	{
		match self
		{
			expr::Value::Bool(value) => Ok(*value),
			_ =>
			{
				report.error_span(
					"expected boolean",
					span);

				Err(())
			}
		}
	}


	pub fn expect_string(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<&ExprString, ()>
	{
		match self
		{
			expr::Value::String(value) => Ok(value),
			_ =>
			{
				report.error_span(
					"expected string",
					span);

				Err(())
			}
		}
	}
}


impl ExprString
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