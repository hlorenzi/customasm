use crate::*;


#[derive(Clone, Debug)]
pub enum Expr
{
	Literal(diagn::Span, Value),
	Variable(diagn::Span, String),
	NestingLevel {
		span: diagn::Span,
		nesting_level: usize,
	},
	MemberAccess {
		span: diagn::Span,
		lhs: Box<Expr>,
		member_name: String,
	},
	UnaryOp(diagn::Span, diagn::Span, UnaryOp, Box<Expr>),
	BinaryOp(diagn::Span, diagn::Span, BinaryOp, Box<Expr>, Box<Expr>),
	TernaryOp(diagn::Span, Box<Expr>, Box<Expr>, Box<Expr>),
	Slice(diagn::Span, diagn::Span, Box<Expr>, Box<Expr>, Box<Expr>),
	SliceShort(diagn::Span, diagn::Span, Box<Expr>, Box<Expr>),
	Block(diagn::Span, Vec<Expr>),
	Call(diagn::Span, Box<Expr>, Vec<Expr>),
	Asm(diagn::Span, asm::AstTopLevel),
}


#[derive(Clone, Debug)]
pub enum Value
{
	Unknown(ValueMetadata),
	FailedConstraint(ValueMetadata, diagn::Message),
	Void(ValueMetadata),
	Integer(ValueMetadata, util::BigInt),
	String(ValueMetadata, ExprString),
	Bool(ValueMetadata, bool),
	ExprBuiltInFunction(ValueMetadata, String),
	AsmBuiltInFunction(ValueMetadata, String),
	Function(ValueMetadata, util::ItemRef<asm::Function>),
}


#[derive(Clone, Debug)]
pub struct ValueMetadata
{
	pub symbol_ref: Option<util::ItemRef<asm::Symbol>>,
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
		Expr::Literal(diagn::Span::new_dummy(), Value::make_bool(false))
	}

	
	pub fn span(&self) -> diagn::Span
	{
		match self
		{
			&Expr::Literal      (span, ..) => span,
			&Expr::Variable     (span, ..) => span,
			&Expr::NestingLevel { span, .. } => span,
			&Expr::MemberAccess { span, .. } => span,
			&Expr::UnaryOp      (span, ..) => span,
			&Expr::BinaryOp     (span, ..) => span,
			&Expr::TernaryOp    (span, ..) => span,
			&Expr::Slice        (span, ..) => span,
			&Expr::SliceShort   (span, ..) => span,
			&Expr::Block        (span, ..) => span,
			&Expr::Call         (span, ..) => span,
			&Expr::Asm          (span, ..) => span,
		}
	}
}


impl Value
{
	pub fn type_name(&self) -> &str
	{
		match self
		{
			Value::Unknown(..) => "unknown",
			Value::FailedConstraint(..) => "failed constraint",
			Value::Void(..) => "void",
			Value::Integer(..) => "integer",
			Value::String(..) => "string",
			Value::Bool(..) => "bool",
			Value::ExprBuiltInFunction(..) => "built-in function",
			Value::AsmBuiltInFunction(..) => "built-in function",
			Value::Function(..) => "function",
		}
	}


	pub fn is_unknown(&self) -> bool
	{
		match self
		{
			Value::Unknown(_) => true,
			_ => false,
		}
	}


	pub fn should_propagate(&self) -> bool
	{
		match self
		{
			Value::Unknown(_) => true,
			Value::FailedConstraint(_, _) => true,
			_ => false,
		}
	}

	
	pub fn make_literal(&self) -> Expr
	{
		Expr::Literal(diagn::Span::new_dummy(), self.clone())
	}


	pub fn make_unknown() -> Value
	{
		Value::Unknown(Value::make_metadata())
	}


	pub fn make_void() -> Value
	{
		Value::Void(Value::make_metadata())
	}


	pub fn make_integer<T: Into<util::BigInt>>(value: T) -> Value
	{
		Value::Integer(
			Value::make_metadata(),
			value.into())
	}


	pub fn make_bool(value: bool) -> Value
	{
		Value::Bool(
			Value::make_metadata(),
			value)
	}


	pub fn make_string<T: Into<String>, S: Into<String>>(value: T, encoding: S) -> Value
	{
		Value::String(
			Value::make_metadata(),
			ExprString {
				utf8_contents: value.into(),
				encoding: encoding.into(),
			})
	}

	
	pub fn make_metadata() -> ValueMetadata
	{
		ValueMetadata {
			symbol_ref: None,
		}
	}


	pub fn get_metadata(&self) -> &ValueMetadata
	{
		match self
		{
			Value::Unknown(meta, ..) => meta,
			Value::FailedConstraint(meta, ..) => meta,
			Value::Void(meta, ..) => meta,
			Value::Integer(meta, ..) => meta,
			Value::String(meta, ..) => meta,
			Value::Bool(meta, ..) => meta,
			Value::ExprBuiltInFunction(meta, ..) => meta,
			Value::AsmBuiltInFunction(meta, ..) => meta,
			Value::Function(meta, ..) => meta,
		}
	}


	pub fn get_mut_metadata(&mut self) -> &mut ValueMetadata
	{
		match self
		{
			Value::Unknown(meta, ..) => meta,
			Value::FailedConstraint(meta, ..) => meta,
			Value::Void(meta, ..) => meta,
			Value::Integer(meta, ..) => meta,
			Value::String(meta, ..) => meta,
			Value::Bool(meta, ..) => meta,
			Value::ExprBuiltInFunction(meta, ..) => meta,
			Value::AsmBuiltInFunction(meta, ..) => meta,
			Value::Function(meta, ..) => meta,
		}
	}


	pub fn get_bigint(&self) -> Option<util::BigInt>
	{
		match &self
		{
			&Value::Integer(_, bigint) => Some(bigint.clone()),
			&Value::String(_, s) => Some(s.to_bigint()),
			_ => None,
		}
	}


	pub fn coallesce_to_integer<'a>(
		&'a self)
		-> std::borrow::Cow<'a, expr::Value>
	{
		match self
		{
			Value::String(_, s) =>
				std::borrow::Cow::Owned(
					expr::Value::make_integer(s.to_bigint())),

			_ => std::borrow::Cow::Borrowed(self),
		}
	}


	pub fn unwrap_bigint(
		&self)
		-> &util::BigInt
	{
		match &self
		{
			Value::Integer(_, bigint) => bigint,
			_ => panic!(),
		}
	}


	pub fn expect_bigint(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<&util::BigInt, ()>
	{
		match &self
		{
			Value::Integer(_, bigint) => Ok(bigint),

			Value::Unknown(_) =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					format!(
						"expected integer, got {}",
						self.type_name()),
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
			Value::Integer(_, bigint) => Ok(bigint),

			Value::Unknown(_) =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					format!(
						"expected integer, got {}",
						self.type_name()),
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
					format!(
						"expected integer with definite size, got {}",
						self.type_name()),
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
			format!(
				"expected integer-like value with definite size, got {}",
				self.type_name()),
			span);

		Err(())
	}


	pub fn expect_error_or_bigint(
		self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<expr::Value, ()>
	{
		let coallesced = self.coallesce_to_integer();

		match coallesced.as_ref()
		{
			expr::Value::Unknown(_) |
			expr::Value::FailedConstraint(_, _) =>
				Ok(coallesced.into_owned()),

			expr::Value::Integer(_, _) =>
				Ok(coallesced.into_owned()),

			_ =>
			{
				report.error_span(
					format!(
						"expected integer, got {}",
						coallesced.type_name()),
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
		let coallesced = self.coallesce_to_integer();

		match coallesced.as_ref()
		{
			expr::Value::Unknown(_) |
			expr::Value::FailedConstraint(_, _) =>
				Ok(coallesced.into_owned()),

			expr::Value::Integer(_, bigint)
				if bigint.size.is_some() =>
				Ok(expr::Value::make_integer(bigint.to_owned())),

			_ =>
			{
				report.error_span(
					format!(
						"expected integer with definite size, got {}",
						coallesced.type_name()),
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
		let coallesced = self.coallesce_to_integer();

		match coallesced.as_ref()
		{
			expr::Value::Unknown(_) |
			expr::Value::FailedConstraint(_, _) =>
				Ok(coallesced.into_owned()),

			expr::Value::Bool(_, _) =>
				Ok(coallesced.into_owned()),

			_ =>
			{
				report.error_span(
					format!(
						"expected boolean, got {}",
						coallesced.type_name()),
					span);

				Err(())
			}
		}
	}


	pub fn as_usize(&self) -> Option<usize>
	{
		match &self
		{
			Value::Integer(_, bigint) =>
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
		match &self
		{
			Value::Integer(_, bigint) =>
				bigint.checked_into::<usize>(
					report,
					span),

			Value::Unknown(_) =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					format!(
						"expected non-negative integer, got {}",
						self.type_name()),
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
			expr::Value::Unknown(_) |
			expr::Value::FailedConstraint(_, _) =>
				Ok(self.clone()),
				
			expr::Value::Integer(_, bigint) =>
			{
				bigint.checked_into::<usize>(
					report,
					span)?;

				Ok(self.clone())
			}

			_ =>
			{
				report.error_span(
					format!(
						"expected non-negative integer, got {}",
						self.type_name()),
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
		match &self
		{
			Value::Integer(_, bigint) =>
			{
				bigint.checked_into_nonzero_usize(
					report,
					span)
			}

			Value::Unknown(_) =>
			{
				report.error_span(
					"value is unknown",
					span);

				Err(())
			}

			_ =>
			{
				report.error_span(
					format!(
						"expected positive integer, got {}",
						self.type_name()),
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
		match &self
		{
			expr::Value::Bool(_, value) => Ok(*value),

			_ =>
			{
				report.error_span(
					format!(
						"expected boolean, got {}",
						self.type_name()),
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
		match &self
		{
			expr::Value::String(_, value) => Ok(value),

			_ =>
			{
				report.error_span(
					format!(
						"expected string, got {}",
						self.type_name()),
					span);

				Err(())
			}
		}
	}
}


impl std::cmp::PartialEq for Value
{
	fn eq(&self, other: &Value) -> bool
	{
		match (self, other)
		{
			(Value::Unknown(_),
				Value::Unknown(_)) => true,

			(Value::FailedConstraint(_, _),
				Value::FailedConstraint(_, _)) => false,

			(Value::Void(_),
				Value::Void(_)) => true,
				
			(Value::Integer(_, a),
				Value::Integer(_, b)) => a == b,
				
			(Value::Bool(_, a),
				Value::Bool(_, b)) => a == b,
				
			(Value::String(_, a),
				Value::String(_, b)) => a == b,
				
			(Value::ExprBuiltInFunction(_, a),
				Value::ExprBuiltInFunction(_, b)) => a == b,
				
			(Value::AsmBuiltInFunction(_, a),
				Value::AsmBuiltInFunction(_, b)) => a == b,
				
			(Value::Function(_, a),
				Value::Function(_, b)) => a == b,

			_ => false,
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