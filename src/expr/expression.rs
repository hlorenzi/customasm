use crate::*;


#[derive(Clone, Debug)]
pub enum Expr
{
	Literal(diagn::Span, Value),
	Variable(diagn::Span, String),
	StructInit {
		span: diagn::Span,
		members_init: Vec<ExprStructMemberInit>,
	},
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
pub struct ExprStructMemberInit
{
	pub span: diagn::Span,
	pub name: String,
	pub value: Expr,
}


#[derive(Clone, Debug)]
pub enum Value
{
	Unknown(ValueMetadata),
	FailedConstraint(ValueMetadata, diagn::Message),
	Void(ValueMetadata),
	Integer(ValueMetadata, util::BigInt),
	Bool(ValueMetadata, bool),
	Struct(ValueMetadata, ValueStruct),
	ExprBuiltinFn(ValueMetadata, expr::ExprBuiltinFn),
	AsmBuiltinFn(ValueMetadata, asm::AsmBuiltinFn),
	Function(ValueMetadata, util::ItemRef<asm::Function>),
	Bankdef(ValueMetadata, util::ItemRef<asm::Bankdef>),
}


#[derive(Copy, Clone, Debug)]
pub struct ValueMetadata
{
	pub is_guess: bool,
	pub symbol_ref: Option<util::ItemRef<asm::Symbol>>,
	pub bank_ref: Option<util::ItemRef<asm::Bankdef>>,
}


#[derive(Clone, Debug, PartialEq)]
pub struct ValueStruct
{
	pub members: Vec<ValueStructMember>,
}


#[derive(Clone, Debug, PartialEq)]
pub struct ValueStructMember
{
	pub name: String,
	pub value: Value,
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
	pub fn span(&self) -> diagn::Span
	{
		match self
		{
			&Expr::Literal      (span, ..) => span,
			&Expr::Variable     (span, ..) => span,
			&Expr::StructInit   { span, .. } => span,
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


	pub fn returned_value_span(&self) -> diagn::Span
	{
		match self
		{
			&expr::Expr::Block(span, ref exprs) =>
			{
				match exprs.last()
				{
					None => span,
					Some(expr) => expr.returned_value_span()
				}
			}
			
			_ => self.span()
		}
	}
}


impl ValueMetadata
{
	pub fn new() -> ValueMetadata
	{
		ValueMetadata {
			is_guess: true,
			symbol_ref: None,
			bank_ref: None,
		}
	}


	pub fn is_guess(&self) -> bool
	{
		self.is_guess
	}


	pub fn statically_known(mut self) -> ValueMetadata
	{
		self.mark_statically_known();
		self
	}


	pub fn mark_statically_known(&mut self)
	{
		self.is_guess = false;
	}


	pub fn mark_statically_known_if(&mut self, resolved: bool)
	{
		self.is_guess = !resolved;
	}


	pub fn mark_guess(&mut self)
	{
		self.is_guess = true;
	}


	pub fn mark_derived_from(&mut self, other: &ValueMetadata)
	{
		self.is_guess |= other.is_guess;
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
			Value::Bool(..) => "bool",
			Value::Struct(..) => "struct",
			Value::ExprBuiltinFn(..) => "built-in function",
			Value::AsmBuiltinFn(..) => "built-in function",
			Value::Function(..) => "function",
			Value::Bankdef(..) => "bankdef",
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


	pub fn is_stable(&self, previous: &expr::Value) -> bool
	{
		!self.is_unknown() && !previous.is_unknown() && self == previous
	}

	
	pub fn make_literal(&self) -> Expr
	{
		Expr::Literal(diagn::Span::new_dummy(), self.clone())
	}


	pub fn make_unknown() -> Value
	{
		Value::Unknown(ValueMetadata::new())
	}


	pub fn make_void() -> Value
	{
		Value::Void(ValueMetadata::new())
	}


	pub fn make_integer<T: Into<util::BigInt>>(value: T) -> Value
	{
		Value::Integer(
			ValueMetadata::new(),
			value.into())
	}


	pub fn make_maybe_integer<T: Into<util::BigInt>>(maybe_value: Option<T>) -> Value
	{
		if let Some(value) = maybe_value
		{
			Value::Integer(
				ValueMetadata::new(),
				value.into())
		}
		else
		{
			Value::make_void()
		}
	}


	pub fn make_bool(value: bool) -> Value
	{
		Value::Bool(
			ValueMetadata::new(),
			value)
	}


	pub fn make_struct(value: ValueStruct) -> Value
	{
		Value::Struct(
			ValueMetadata::new(),
			value)
	}


	pub fn make_bankdef(value: util::ItemRef<asm::Bankdef>) -> Value
	{
		Value::Bankdef(
			ValueMetadata::new(),
			value)
	}

	
	#[must_use]
	pub fn with_metadata(mut self, metadata: ValueMetadata) -> Value
	{
		*self.get_mut_metadata() = metadata;
		self
	}


	#[must_use]
	pub fn statically_known(mut self) -> Value
	{
		self.get_mut_metadata().mark_statically_known();
		self
	}


	#[must_use]
	pub fn statically_known_if(mut self, resolved: bool) -> Value
	{
		self.get_mut_metadata().mark_statically_known_if(resolved);
		self
	}


	#[must_use]
	pub fn as_guess(mut self) -> Value
	{
		self.get_mut_metadata().mark_guess();
		self
	}


	pub fn mark_guess(&mut self)
	{
		self.get_mut_metadata().mark_guess();
	}


	#[must_use]
	pub fn derived_from(mut self, other: &Value) -> Value
	{
		self.mark_derived_from(other);
		self
	}


	pub fn mark_derived_from(&mut self, other: &Value)
	{
		self.get_mut_metadata().mark_derived_from(other.get_metadata());
	}


	#[must_use]
	pub fn with_symbol_ref(mut self, symbol_ref: util::ItemRef<asm::Symbol>) -> Value
	{
		self.get_mut_metadata().symbol_ref = Some(symbol_ref);
		self
	}


	#[must_use]
	pub fn with_bank_ref(mut self, bank_ref: util::ItemRef<asm::Bankdef>) -> Value
	{
		self.get_mut_metadata().bank_ref = Some(bank_ref);
		self
	}


	pub fn is_guess(&self) -> bool
	{
		self.is_unknown() || self.get_metadata().is_guess
	}


	pub fn get_metadata(&self) -> &ValueMetadata
	{
		match self
		{
			Value::Unknown(meta, ..) => meta,
			Value::FailedConstraint(meta, ..) => meta,
			Value::Void(meta, ..) => meta,
			Value::Integer(meta, ..) => meta,
			Value::Bool(meta, ..) => meta,
			Value::Struct(meta, ..) => meta,
			Value::ExprBuiltinFn(meta, ..) => meta,
			Value::AsmBuiltinFn(meta, ..) => meta,
			Value::Function(meta, ..) => meta,
			Value::Bankdef(meta, ..) => meta,
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
			Value::Bool(meta, ..) => meta,
			Value::Struct(meta, ..) => meta,
			Value::ExprBuiltinFn(meta, ..) => meta,
			Value::AsmBuiltinFn(meta, ..) => meta,
			Value::Function(meta, ..) => meta,
			Value::Bankdef(meta, ..) => meta,
		}
	}


	pub fn get_bigint(&self) -> Option<util::BigInt>
	{
		match &self
		{
			&Value::Integer(_, bigint) => Some(bigint.clone()),
			_ => None,
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
		if let Some(bigint) = self.get_bigint()
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
		match self
		{
			expr::Value::Unknown(_) |
			expr::Value::FailedConstraint(_, _) =>
				Ok(self),

			expr::Value::Integer(_, _) =>
				Ok(self),

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


	pub fn expect_error_or_sized_bigint(
		self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<expr::Value, ()>
	{
		match self
		{
			expr::Value::Unknown(_) |
			expr::Value::FailedConstraint(_, _) =>
				Ok(self),

			expr::Value::Integer(_, ref bigint)
				if bigint.size.is_some() =>
				Ok(self),

			_ =>
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


	pub fn expect_error_or_bool(
		self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<expr::Value, ()>
	{
		match self
		{
			expr::Value::Unknown(_) |
			expr::Value::FailedConstraint(_, _) =>
				Ok(self),

			expr::Value::Bool(_, _) =>
				Ok(self),

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
		-> Result<String, ()>
	{
		if let Value::Integer(_, bigint) = self
		{
			return Ok(bigint.as_string());
		}

		report.error_span(
			format!(
				"expected string, got {}",
				self.type_name()),
			span);

		Err(())
	}


	pub fn convert_string_encoding(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span,
		to_encoding: &str)
		-> Result<expr::Value, ()>
	{
		let utf8_contents = self.expect_string(report, span)?;

		match to_encoding
		{
			"utf8" =>
			{
				Ok(expr::Value::make_integer(
						util::BigInt::from_bytes_be(utf8_contents.as_bytes()))
					.statically_known()
					.derived_from(&self))
			}
			"utf16be" =>
			{
				let units = utf8_contents.encode_utf16();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit >> 8) & 0xff) as u8);
					bytes.push(((unit >> 0) & 0xff) as u8);
				}
					
				Ok(expr::Value::make_integer(
						util::BigInt::from_bytes_be(&bytes[..]))
					.statically_known()
					.derived_from(&self))
			}
			"utf16le" =>
			{
				let units = utf8_contents.encode_utf16();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit >> 0) & 0xff) as u8);
					bytes.push(((unit >> 8) & 0xff) as u8);
				}
					
				Ok(expr::Value::make_integer(
						util::BigInt::from_bytes_be(&bytes[..]))
					.statically_known()
					.derived_from(&self))
			}
			"utf32be" =>
			{
				let units = utf8_contents.chars();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit as u32 >> 24) & 0xff) as u8);
					bytes.push(((unit as u32 >> 16) & 0xff) as u8);
					bytes.push(((unit as u32 >> 8) & 0xff) as u8);
					bytes.push(((unit as u32 >> 0) & 0xff) as u8);
				}
					
				Ok(expr::Value::make_integer(
						util::BigInt::from_bytes_be(&bytes[..]))
					.statically_known()
					.derived_from(&self))
			}
			"utf32le" =>
			{
				let units = utf8_contents.chars();
				let mut bytes = Vec::new();
				for unit in units
				{
					bytes.push(((unit as u32 >> 0) & 0xff) as u8);
					bytes.push(((unit as u32 >> 8) & 0xff) as u8);
					bytes.push(((unit as u32 >> 16) & 0xff) as u8);
					bytes.push(((unit as u32 >> 24) & 0xff) as u8);
				}
					
				Ok(expr::Value::make_integer(
						util::BigInt::from_bytes_be(&bytes[..]))
					.statically_known()
					.derived_from(&self))
			}
			"ascii" =>
			{
				let units = utf8_contents.chars();
				let bytes = units.map(|c| {
					if c as u32 >= 0x100
					{
						0x00
					}
					else
					{
						c as u8
					}
				});
					
				Ok(expr::Value::make_integer(
						util::BigInt::from_bytes_be(&bytes.collect::<Vec<_>>()[..]))
					.statically_known()
					.derived_from(&self))
			}
			_ => panic!("invalid string encoding"),
		}
	}
}


impl std::cmp::PartialEq for Value
{
	fn eq(&self, other: &Value) -> bool
	{
		// Ignore the values' metadata in comparisons.
		match self
		{
			Value::Unknown(_) => match other
			{
				Value::Unknown(_) => true,
				_ => false,
			}

			Value::FailedConstraint(_, _) => match other
			{
				Value::FailedConstraint(_, _) => true,
				_ => false,
			}

			Value::Void(_) => match other
			{
				Value::Void(_) => true,
				_ => false,
			}

			Value::Integer(_, a) => match other
			{
				Value::Integer(_, b) => a == b,
				_ => false,
			}

			Value::Bool(_, a) => match other
			{
				Value::Bool(_, b) => a == b,
				_ => false,
			}

			Value::Struct(_, a) => match other
			{
				Value::Struct(_, b) => a == b,
				_ => false,
			}

			Value::ExprBuiltinFn(_, a) => match other
			{
				Value::ExprBuiltinFn(_, b) => a == b,
				_ => false,
			}

			Value::AsmBuiltinFn(_, a) => match other
			{
				Value::AsmBuiltinFn(_, b) => a == b,
				_ => false,
			}

			Value::Function(_, a) => match other
			{
				Value::Function(_, b) => a == b,
				_ => false,
			}

			Value::Bankdef(_, a) => match other
			{
				Value::Bankdef(_, b) => a == b,
				_ => false,
			}
		}
	}
}


impl std::fmt::Display for expr::Value
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
		let mut write_meta = |meta: &ValueMetadata| {
			write!(f, "{}", if !meta.is_guess { "✅" } else { "" })
		};

		match self
		{
			Value::Unknown(meta) => {
				write_meta(meta)?;
				write!(f, "Unknown")
			}
			Value::FailedConstraint(meta, ..) => {
				write_meta(meta)?;
				write!(f, "FailedConstraint")
			}
			Value::Void(meta) => {
				write_meta(meta)?;
				write!(f, "Void")
			}
			Value::Integer(meta, value) => {
				write_meta(meta)?;
				write!(f, "Integer({:?})", value)
			}
			Value::Bool(meta, value) => {
				write_meta(meta)?;
				write!(f, "Bool({:?})", value)
			}
			Value::Struct(meta, value) => {
				write_meta(meta)?;
				write!(f, "Struct({:?})", value)
			}
			Value::ExprBuiltinFn(meta, value) => {
				write_meta(meta)?;
				write!(f, "ExprBuiltinFn({:?})", value)
			}
			Value::AsmBuiltinFn(meta, value) => {
				write_meta(meta)?;
				write!(f, "AsmBuiltinFn({:?})", value)
			}
			Value::Function(meta, value) => {
				write_meta(meta)?;
				write!(f, "Function({:?})", value)
			}
			Value::Bankdef(meta, value) => {
				write_meta(meta)?;
				write!(f, "Bankdef({:?})", value)
			}
		}
    }
}