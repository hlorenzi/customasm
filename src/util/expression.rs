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
	And(Box<Expression>, Box<Expression>),
	Or(Box<Expression>, Box<Expression>),
	Xor(Box<Expression>, Box<Expression>),
	
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
			&ExpressionTerm::And(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Or(ref lhs, ref rhs)  => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Xor(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			
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
			
			&ExpressionTerm::Add(ref lhs, ref rhs) => ExpressionValue::add(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Sub(ref lhs, ref rhs) => ExpressionValue::sub(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Mul(ref lhs, ref rhs) => ExpressionValue::mul(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Div(ref lhs, ref rhs) => ExpressionValue::div(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Shl(ref lhs, ref rhs) => ExpressionValue::shl(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Shr(ref lhs, ref rhs) => ExpressionValue::shr(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::And(ref lhs, ref rhs) => ExpressionValue::and(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Or(ref lhs, ref rhs)  => ExpressionValue::or(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Xor(ref lhs, ref rhs) => ExpressionValue::xor(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Eq(ref lhs, ref rhs) => ExpressionValue::eq(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Ne(ref lhs, ref rhs) => ExpressionValue::ne(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Lt(ref lhs, ref rhs) => ExpressionValue::ls(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Gt(ref lhs, ref rhs) => ExpressionValue::gt(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Le(ref lhs, ref rhs) => ExpressionValue::le(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			&ExpressionTerm::Ge(ref lhs, ref rhs) => ExpressionValue::ge(&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Slice(ref expr, left, right) => Ok(ExpressionValue::ArbitraryPrecision(try!(expr.resolve(check_name)).as_bitvec().slice(left, right)))
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
	

	fn perform_arith<F>(a: &ExpressionValue, b: &ExpressionValue, span: &Span, op: F) -> Result<ExpressionValue, Error>
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
	

	fn perform_rel<F>(a: &ExpressionValue, b: &ExpressionValue, span: &Span, op: F) -> Result<ExpressionValue, Error>
	where F: Fn(i64, i64) -> bool
	{
		if let Some(ma) = a.as_i64()
		{
			if let Some(mb) = b.as_i64()
				{ return Ok(ExpressionValue::Boolean(op(ma, mb))); }
		}
		
		Err(Error::new_with_span("invalid operands", span.clone()))
	}
	

	pub fn add(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| a.checked_add(b))
	}
	

	pub fn sub(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| a.checked_sub(b))
	}
	

	pub fn mul(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| a.checked_mul(b))
	}
	

	pub fn div(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| a.checked_div(b))
	}
	

	pub fn shl(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| a.checked_shl(b as u32))
	}
	

	pub fn shr(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| a.checked_shr(b as u32))
	}
	

	pub fn and(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| Some(a & b))
	}
	

	pub fn or(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| Some(a | b))
	}
	

	pub fn xor(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_arith(a, b, span, |a, b| Some(a ^ b))
	}
	

	pub fn eq(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_rel(a, b, span, |a, b| a == b)
	}
	

	pub fn ne(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_rel(a, b, span, |a, b| a != b)
	}
	

	pub fn ls(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_rel(a, b, span, |a, b| a < b)
	}
	

	pub fn gt(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_rel(a, b, span, |a, b| a > b)
	}
	

	pub fn le(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_rel(a, b, span, |a, b| a <= b)
	}
	

	pub fn ge(a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	{
		ExpressionValue::perform_rel(a, b, span, |a, b| a >= b)
	}
}

	
impl<'p, 'f, 'tok> ExpressionParser<'p, 'f, 'tok>
{
	pub fn parse(&mut self) -> Result<Expression, Error>
	{
		self.parse_relational_term()
	}
	
	
	fn parse_relational_term(&mut self) -> Result<Expression, Error>
	{
		let mut lhs = try!(self.parse_addition_term());
		
		loop
		{
			let op = 
				if self.parser.current().is_operator("==")
					{ "==" }
				else if self.parser.current().is_operator("!=")
					{ "!=" }
				else if self.parser.current().is_operator("<=")
					{ "<=" }
				else if self.parser.current().is_operator(">=")
					{ ">=" }
				else if self.parser.current().is_operator("<")
					{ "<" }
				else if self.parser.current().is_operator(">")
					{ ">" }
				else
					{ break; };
			
			self.parser.advance();
			
			let rhs = try!(self.parse_addition_term());
			
			let span = lhs.span.join(&rhs.span);
			
			let term =
				match op
				{
					"==" => ExpressionTerm::Eq(Box::new(lhs), Box::new(rhs)),
					"!=" => ExpressionTerm::Ne(Box::new(lhs), Box::new(rhs)),
					"<=" => ExpressionTerm::Le(Box::new(lhs), Box::new(rhs)),
					">=" => ExpressionTerm::Ge(Box::new(lhs), Box::new(rhs)),
					"<" => ExpressionTerm::Lt(Box::new(lhs), Box::new(rhs)),
					">" => ExpressionTerm::Gt(Box::new(lhs), Box::new(rhs)),
					_ => unreachable!()
				};
			
			lhs = Expression
			{
				span: span,
				term: term
			};
		}
		
		Ok(lhs)
	}

	
	fn parse_addition_term(&mut self) -> Result<Expression, Error>
	{
		let mut lhs = try!(self.parse_multiplication_term());
		
		loop
		{
			let is_add = 
				if self.parser.current().is_operator("+")
					{ true }
				else if self.parser.current().is_operator("-")
					{ false }
				else
					{ break; };
			
			self.parser.advance();
			
			let rhs = try!(self.parse_multiplication_term());
			
			let span = lhs.span.join(&rhs.span);
			
			let term =
				if is_add
					{ ExpressionTerm::Add(Box::new(lhs), Box::new(rhs)) }
				else
					{ ExpressionTerm::Sub(Box::new(lhs), Box::new(rhs)) };
			
			lhs = Expression
			{
				span: span,
				term: term
			};
		}
		
		Ok(lhs)
	}

	
	fn parse_multiplication_term(&mut self) -> Result<Expression, Error>
	{
		let mut lhs = try!(self.parse_bitop_term());
		
		loop
		{
			let is_mul = 
				if self.parser.current().is_operator("*")
					{ true }
				else if self.parser.current().is_operator("/")
					{ false }
				else
					{ break; };
			
			self.parser.advance();
			
			let rhs = try!(self.parse_bitop_term());
			
			let span = lhs.span.join(&rhs.span);
			
			let term =
				if is_mul
					{ ExpressionTerm::Mul(Box::new(lhs), Box::new(rhs)) }
				else
					{ ExpressionTerm::Div(Box::new(lhs), Box::new(rhs)) };
			
			lhs = Expression
			{
				span: span,
				term: term
			};
		}
		
		Ok(lhs)
	}
	
	
	fn parse_bitop_term(&mut self) -> Result<Expression, Error>
	{
		let mut lhs = try!(self.parse_slice_term());
		
		loop
		{
			let op = 
				if self.parser.current().is_operator("<<")
					{ "<<" }
				else if self.parser.current().is_operator(">>")
					{ ">>" }
				else if self.parser.current().is_operator("&")
					{ "&" }
				else if self.parser.current().is_operator("|")
					{ "|" }
				else if self.parser.current().is_operator("^")
					{ "^" }
				else
					{ break; };
			
			self.parser.advance();
			
			let rhs = try!(self.parse_slice_term());
			
			let span = lhs.span.join(&rhs.span);
			
			let term =
				match op
				{
					"<<" => ExpressionTerm::Shl(Box::new(lhs), Box::new(rhs)),
					">>" => ExpressionTerm::Shr(Box::new(lhs), Box::new(rhs)),
					"&" => ExpressionTerm::And(Box::new(lhs), Box::new(rhs)),
					"|" => ExpressionTerm::Or(Box::new(lhs), Box::new(rhs)),
					"^" => ExpressionTerm::Xor(Box::new(lhs), Box::new(rhs)),
					_ => unreachable!()
				};
			
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
			let leftmost_bit = try!(self.parser.expect_number()).number_usize();
			try!(self.parser.expect_operator(":"));
			let rightmost_bit = try!(self.parser.expect_number()).number_usize();
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
			let token = try!(self.parser.expect_identifier()).clone();
			let ident = token.identifier().clone();
			
			if let Some(check_var) = self.check_var
			{
				if !check_var(&ident)
					{ return Err(Error::new_with_span(format!("unknown variable `{}`", ident), token.span.clone())); }
			}
			
			Ok(Expression
			{
				span: token.span.clone(),
				term: ExpressionTerm::GlobalVariable(ident)
			})
		}
		
		else if self.parser.current().is_operator("'") && self.parser.next(1).is_identifier()
		{
			let start_span = self.parser.current().span.clone();
			self.parser.advance();
			
			let token = try!(self.parser.expect_identifier()).clone();
			let ident = token.identifier().clone();
			
			Ok(Expression
			{
				span: token.span.join(&start_span),
				term: ExpressionTerm::LocalVariable(ident)
			})
		}
		
		else if self.parser.current().is_number()
		{
			let size = 
				if self.parser.next(1).is_operator("'")
				{
					let size = try!(self.parser.expect_number()).number_usize();
					try!(self.parser.expect_operator("'"));
					size
				}
				else
					{ 0 };
					
			let token = try!(self.parser.expect_number()).clone();
			let (radix, value_str) = token.number();
			
			match BitVec::new_from_str_trimmed(radix, value_str)
			{
				Err(msg) => Err(self.parser.make_error(msg, &token.span)),
				Ok(mut bitvec) =>
				{
					if size != 0
					{
						if bitvec.len() > size
							{ return Err(Error::new_with_span("value does not fit given size", token.span.clone())); }
					
						bitvec.zero_extend(size);
					}
				
					Ok(Expression
					{
						span: token.span.clone(),
						term: ExpressionTerm::LiteralUInt(bitvec)
					})
				}
			}
		}
		
		else
			{ Err(self.parser.make_error("expected expression", &self.parser.current().span)) }
	}
}