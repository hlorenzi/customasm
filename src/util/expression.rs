use util::bitvec::BitVec;
use util::error::Error;
use util::parser::Parser;
use util::tokenizer::Span;


pub struct Expression
{
	pub span: Span,
	pub term: ExpressionTerm
}


pub enum ExpressionTerm
{
	LiteralUInt(BitVec),
	GlobalVariable(String),
	LocalVariable(String),
	
	Add(Box<Expression>, Box<Expression>),
	Sub(Box<Expression>, Box<Expression>),
	Mul(Box<Expression>, Box<Expression>),
	Div(Box<Expression>, Box<Expression>),
	Shl(Box<Expression>, Box<Expression>),
	Shr(Box<Expression>, Box<Expression>),
	BitAnd(Box<Expression>, Box<Expression>),
	BitOr(Box<Expression>, Box<Expression>),
	BitXor(Box<Expression>, Box<Expression>),
	
	And(Box<Expression>, Box<Expression>),
	Or(Box<Expression>, Box<Expression>),
	
	Eq(Box<Expression>, Box<Expression>),
	Ne(Box<Expression>, Box<Expression>),
	Lt(Box<Expression>, Box<Expression>),
	Gt(Box<Expression>, Box<Expression>),
	Le(Box<Expression>, Box<Expression>),
	Ge(Box<Expression>, Box<Expression>),
	
	Slice(Box<Expression>, usize, usize)
}


pub enum ExpressionName<'s>
{
	GlobalVariable(&'s str),
	LocalVariable(&'s str)
}


pub enum ExpressionValue
{
	ArbitraryPrecision(BitVec),
	MachinePrecision(i64),
	Boolean(bool)
}


struct ExpressionParser<'p, 'f: 'p, 'tok: 'p>
{
	parser: &'p mut Parser<'f, 'tok>,
	check_var: Option<&'p Fn(&str) -> bool>
}


impl Expression
{
	pub fn new_by_parsing<'p, 'f: 'p, 'tok: 'p>(parser: &'p mut Parser<'f, 'tok>) -> Result<Expression, Error>
	{
		let mut expr_parser = ExpressionParser { parser: parser, check_var: None };
		expr_parser.parse()
	}
	
	
	pub fn new_by_parsing_checked<'p, 'f: 'p, 'tok: 'p>(
		parser: &'p mut Parser<'f, 'tok>,
		check_var: &Fn(&str) -> bool) -> Result<Expression, Error>
	{
		let mut expr_parser = ExpressionParser { parser: parser, check_var: Some(check_var) };
		expr_parser.parse()
	}
	
	
	pub fn new_literal_integer(value: usize, span: Span) -> Expression
	{
		Expression
		{
			span: span,
			term: ExpressionTerm::LiteralUInt(BitVec::new_from_usize(value))
		}
	}
	
	
	pub fn can_resolve<F>(&self, check_name: &F) -> Result<bool, Error>
	where F: Fn(ExpressionName, &Span) -> Result<bool, Error>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(_)           => Ok(true),
			&ExpressionTerm::GlobalVariable(ref name) => check_name(ExpressionName::GlobalVariable(&name), &self.span),
			&ExpressionTerm::LocalVariable(ref name)  => check_name(ExpressionName::LocalVariable(&name), &self.span),
			
			&ExpressionTerm::Add(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Sub(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Mul(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Div(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Shl(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Shr(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::BitAnd(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::BitOr(ref lhs, ref rhs)  => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::BitXor(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			
			&ExpressionTerm::And(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Or(ref lhs, ref rhs)  => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			
			&ExpressionTerm::Eq(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Ne(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Lt(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Gt(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Le(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Ge(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			
			&ExpressionTerm::Slice(ref expr, _, _) => expr.can_resolve(check_name)
		}
	}
	
	
	pub fn get_explicit_bit_num(&self) -> Option<usize>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(ref bitvec) => Some(bitvec.len()),
			
			&ExpressionTerm::Slice(_, left, right) =>
				if left > right
					{ Some(left - right + 1) }
				else
					{ Some(right - left + 1) },
					
			_ => None
		}
	}
	
	
	pub fn get_minimum_bit_num<F>(&self, check_name: &F) -> Result<Option<usize>, Error>
	where F: Fn(ExpressionName, &Span) -> Result<Option<usize>, Error>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(ref bitvec)  => Ok(Some(bitvec.len())),
			&ExpressionTerm::GlobalVariable(ref name) => check_name(ExpressionName::GlobalVariable(&name), &self.span),				
			&ExpressionTerm::LocalVariable(ref name)  => check_name(ExpressionName::LocalVariable(&name), &self.span),
			
			&ExpressionTerm::Slice(_, left, right) =>
				if left > right
					{ Ok(Some(left - right + 1)) }
				else
					{ Ok(Some(right - left + 1)) },
					
			_ => Ok(None)
		}
	}
	
	
	pub fn resolve<F>(&self, check_name: &F) -> Result<ExpressionValue, Error>
	where F: Fn(ExpressionName, &Span) -> Result<ExpressionValue, Error>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(ref bitvec)  => Ok(ExpressionValue::ArbitraryPrecision(bitvec.clone())),
			&ExpressionTerm::GlobalVariable(ref name) => check_name(ExpressionName::GlobalVariable(&name), &self.span),				
			&ExpressionTerm::LocalVariable(ref name)  => check_name(ExpressionName::LocalVariable(&name), &self.span),
			
			&ExpressionTerm::Add(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| a.checked_add(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Sub(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| a.checked_sub(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Mul(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| a.checked_mul(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Div(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| a.checked_div(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Shl(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| a.checked_shl(b as u32),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Shr(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| a.checked_shr(b as u32),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::BitAnd(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| Some(a & b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::BitOr(ref lhs, ref rhs)  => ExpressionValue::perform_arith(|a, b| Some(a | b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::BitXor(ref lhs, ref rhs) => ExpressionValue::perform_arith(|a, b| Some(a ^ b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
				
			&ExpressionTerm::And(ref lhs, ref rhs) => ExpressionValue::perform_bool(|a, b| a && b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Or(ref lhs, ref rhs)  => ExpressionValue::perform_bool(|a, b| a || b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			
			&ExpressionTerm::Eq(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a == b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Ne(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a != b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Lt(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a < b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Gt(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a > b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Le(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a <= b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Ge(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a >= b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Slice(ref expr, left, right) =>
				Ok(ExpressionValue::ArbitraryPrecision(try!(expr.resolve(check_name)).as_bitvec().slice(left, right)))
		}
	}
}


impl ExpressionValue
{
	pub fn as_bitvec(&self) -> BitVec
	{
		match self
		{
			&ExpressionValue::MachinePrecision(value) => BitVec::new_from_i64(value),
			&ExpressionValue::ArbitraryPrecision(ref bitvec) => bitvec.clone(),
			&ExpressionValue::Boolean(value) => BitVec::new_from_vec(vec![value])
		}
	}
	
	
	pub fn as_i64(&self) -> Option<i64>
	{
		match self
		{
			&ExpressionValue::MachinePrecision(value) => Some(value),
			&ExpressionValue::ArbitraryPrecision(ref bitvec) => bitvec.to_i64(),
			&ExpressionValue::Boolean(_) => None
		}
	}
	
	
	pub fn as_bool(&self) -> Option<bool>
	{
		match self
		{
			&ExpressionValue::Boolean(value) => Some(value),
			_ => None
		}
	}
	

	pub fn perform_arith<F>(op: F, a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	where F: Fn(i64, i64) -> Option<i64>
	{
		if let Some(ma) = a.as_i64()
		{
			if let Some(mb) = b.as_i64()
			{
				match op(ma, mb)
				{
					Some(value) => return Ok(ExpressionValue::MachinePrecision(value)),
					None => return Err(Error::new_with_span("machine-precision overflow", span.clone()))
				}
			}
		}
		
		Err(Error::new_with_span("invalid operands", span.clone()))
	}
	

	pub fn perform_rel<F>(op: F, a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	where F: Fn(i64, i64) -> bool
	{
		if let Some(ma) = a.as_i64()
		{
			if let Some(mb) = b.as_i64()
				{ return Ok(ExpressionValue::Boolean(op(ma, mb))); }
		}
		
		Err(Error::new_with_span("invalid operands", span.clone()))
	}
	

	pub fn perform_bool<F>(op: F, a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	where F: Fn(bool, bool) -> bool
	{
		if let Some(ma) = a.as_bool()
		{
			if let Some(mb) = b.as_bool()
				{ return Ok(ExpressionValue::Boolean(op(ma, mb))); }
		}
		
		Err(Error::new_with_span("invalid operands", span.clone()))
	}
}

	
impl<'p, 'f, 'tok> ExpressionParser<'p, 'f, 'tok>
{
	pub fn parse(&mut self) -> Result<Expression, Error>
	{
		self.parse_binary_op_term(0)
	}
	
	
	fn parse_binary_op_term(&mut self, level: usize) -> Result<Expression, Error>
	{	
		let binary_ops: &[&[(&str, &Fn(Box<Expression>, Box<Expression>) -> ExpressionTerm)]] =
		&[
			&[("||", &ExpressionTerm::Or)],
			&[("&&", &ExpressionTerm::And)],
			&[
				("==", &ExpressionTerm::Eq),
				("!=", &ExpressionTerm::Ne),
				("<",  &ExpressionTerm::Lt),
				(">",  &ExpressionTerm::Gt),
				("<=", &ExpressionTerm::Le),
				(">=", &ExpressionTerm::Ge)
			],
			&[("|",  &ExpressionTerm::BitOr)],
			&[("^",  &ExpressionTerm::BitXor)],
			&[("&",  &ExpressionTerm::BitAnd)],
			&[
				("<<", &ExpressionTerm::Shl),
				(">>", &ExpressionTerm::Shr)
			],
			&[
				("+",  &ExpressionTerm::Add),
				("-",  &ExpressionTerm::Sub)
			],
			&[
				("*",  &ExpressionTerm::Mul),
				("/",  &ExpressionTerm::Div)
			],
		];
		
		if level >= binary_ops.len()
			{ return self.parse_slice_term(); }
	
		let mut lhs = try!(self.parse_binary_op_term(level + 1));
		
		loop
		{
			let mut op_match = None;
			for op in binary_ops[level]
			{
				if self.parser.current().is_operator(op.0)
				{
					op_match = Some(op);
					break;
				}
			}
			
			if op_match.is_none()
				{ break; }
			
			self.parser.advance();
			
			let rhs = try!(self.parse_binary_op_term(level + 1));
			
			let span = lhs.span.join(&rhs.span);
			let term = (op_match.unwrap().1)(Box::new(lhs), Box::new(rhs));
			
			lhs = Expression
			{
				span: span,
				term: term
			};
		}
		
		Ok(lhs)
	}
	
	
	fn parse_slice_term(&mut self) -> Result<Expression, Error>
	{
		let expr = try!(self.parse_leaf_term());
		
		if self.parser.current().is_operator("[")
		{
			self.parser.advance();
			let (leftmost_bit, _) = try!(self.parser.expect_number());
			try!(self.parser.expect_operator(":"));
			let (rightmost_bit, _) = try!(self.parser.expect_number());
			try!(self.parser.expect_operator("]"));
			
			Ok(Expression
			{
				span: expr.span.join(&self.parser.current().span),
				term: ExpressionTerm::Slice(Box::new(expr), leftmost_bit, rightmost_bit)
			})
		}
		else
			{ Ok(expr) }
	}
	
	
	fn parse_leaf_term(&mut self) -> Result<Expression, Error>
	{
		if self.parser.current().is_operator("(")
		{
			self.parser.advance();
			let expr = try!(self.parse());
			try!(self.parser.expect_operator(")"));
			Ok(expr)
		}
	
		else if self.parser.current().is_identifier()
		{
			let (ident, ident_span) = try!(self.parser.expect_identifier());
			
			if let Some(check_var) = self.check_var
			{
				if !check_var(&ident)
					{ return Err(Error::new_with_span(format!("unknown variable `{}`", ident), ident_span)); }
			}
			
			Ok(Expression
			{
				span: ident_span,
				term: ExpressionTerm::GlobalVariable(ident)
			})
		}
		
		else if self.parser.current().is_operator("'") && self.parser.next(1).is_identifier()
		{
			let start_span = self.parser.current().span.clone();
			self.parser.advance();
			
			let (ident, ident_span) = try!(self.parser.expect_identifier());
			
			Ok(Expression
			{
				span: ident_span.join(&start_span),
				term: ExpressionTerm::LocalVariable(ident)
			})
		}
		
		else if self.parser.current().is_number()
		{
			let data_width = 
				if self.parser.next(1).is_operator("'")
				{
					let (data_width, _) = try!(self.parser.expect_number());
					try!(self.parser.expect_operator("'"));
					data_width
				}
				else
					{ 0 };
					
			let (radix, value_str, value_span) = try!(self.parser.expect_number_str());
			
			match BitVec::new_from_str_trimmed(radix, value_str)
			{
				Err(msg) => Err(Error::new_with_span(msg, value_span)),
				Ok(mut bitvec) =>
				{
					if data_width != 0
					{
						if bitvec.len() > data_width
							{ return Err(Error::new_with_span("value does not fit given width", value_span)); }
					
						bitvec.zero_extend(data_width);
					}
				
					Ok(Expression
					{
						span: value_span,
						term: ExpressionTerm::LiteralUInt(bitvec)
					})
				}
			}
		}
		
		else
			{ Err(Error::new_with_span("expected expression", self.parser.current().span.clone())) }
	}
}