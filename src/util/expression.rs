use util::error::Error;
use util::integer::Integer;
use util::parser::Parser;
use util::tokenizer::Span;


pub struct Expression
{
	pub span: Span,
	pub term: ExpressionTerm
}


pub enum ExpressionTerm
{
	LiteralInteger(Integer),
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


#[derive(Clone)]
pub enum ExpressionValue
{
	Integer(Integer),
	Boolean(bool)
}


struct ExpressionParser<'p, 'tok: 'p>
{
	parser: &'p mut Parser<'tok>,
	check_var: Option<&'p Fn(&str) -> bool>
}


impl Expression
{
	pub fn new_by_parsing<'p, 'tok: 'p>(parser: &'p mut Parser<'tok>) -> Result<Expression, Error>
	{
		let mut expr_parser = ExpressionParser { parser: parser, check_var: None };
		expr_parser.parse()
	}
	
	
	pub fn new_by_parsing_checked<'p, 'tok: 'p>(
		parser: &'p mut Parser<'tok>,
		check_var: &Fn(&str) -> bool) -> Result<Expression, Error>
	{
		let mut expr_parser = ExpressionParser { parser: parser, check_var: Some(check_var) };
		expr_parser.parse()
	}
	
	
	pub fn new_literal_integer(value: i64, span: Span) -> Expression
	{
		Expression
		{
			span: span,
			term: ExpressionTerm::LiteralInteger(Integer::new(value))
		}
	}
	
	
	pub fn can_resolve<F>(&self, check_name: &F) -> Result<bool, Error>
	where F: Fn(ExpressionName, &Span) -> Result<bool, Error>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralInteger(_)        => Ok(true),
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
			&ExpressionTerm::LiteralInteger(ref integer) => integer.explicit_width,
			
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
			&ExpressionTerm::LiteralInteger(ref integer)  => Ok(Some(integer.get_minimum_width())),
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
			&ExpressionTerm::LiteralInteger(ref integer)  => Ok(ExpressionValue::Integer(integer.clone())),
			&ExpressionTerm::GlobalVariable(ref name) => check_name(ExpressionName::GlobalVariable(&name), &self.span),				
			&ExpressionTerm::LocalVariable(ref name)  => check_name(ExpressionName::LocalVariable(&name), &self.span),
			
			&ExpressionTerm::Add(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.checked_add(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Sub(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.checked_sub(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Mul(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.checked_mul(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Div(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.checked_div(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Shl(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.checked_shl(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Shr(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.checked_shr(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::BitAnd(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.bit_and(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::BitOr(ref lhs, ref rhs)  => ExpressionValue::perform_binary_arith(|a, b| a.bit_or(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::BitXor(ref lhs, ref rhs) => ExpressionValue::perform_binary_arith(|a, b| a.bit_xor(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
				
			&ExpressionTerm::And(ref lhs, ref rhs) => ExpressionValue::perform_bool(|a, b| a && b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
				
			&ExpressionTerm::Or(ref lhs, ref rhs)  => ExpressionValue::perform_bool(|a, b| a || b,
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			
			&ExpressionTerm::Eq(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a.eq(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Ne(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a.ne(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Lt(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a.lt(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Gt(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a.gt(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Le(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a.le(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Ge(ref lhs, ref rhs) => ExpressionValue::perform_rel(|a, b| a.ge(b),
				&try!(lhs.resolve(check_name)), &try!(rhs.resolve(check_name)), &self.span),
			
			&ExpressionTerm::Slice(ref expr, left, right) => ExpressionValue::perform_unary_arith(|a| Some(a.slice(left, right)),
				&try!(expr.resolve(check_name)), &self.span)
		}
	}
}


impl ExpressionValue
{
	pub fn as_integer(&self) -> Option<Integer>
	{
		match self
		{
			&ExpressionValue::Integer(ref value) => Some(value.clone()),
			_ => None
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
	

	pub fn perform_unary_arith<F>(op: F, a: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	where F: Fn(&Integer) -> Option<Integer>
	{
		if let Some(ma) = a.as_integer()
		{
			match op(&ma)
			{
				Some(value) => return Ok(ExpressionValue::Integer(value)),
				None => return Err(Error::new_with_span("integer overflow", span.clone()))
			}
		}
		
		Err(Error::new_with_span("invalid operands", span.clone()))
	}
	

	pub fn perform_binary_arith<F>(op: F, a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	where F: Fn(&Integer, &Integer) -> Option<Integer>
	{
		if let Some(ma) = a.as_integer()
		{
			if let Some(mb) = b.as_integer()
			{
				match op(&ma, &mb)
				{
					Some(value) => return Ok(ExpressionValue::Integer(value)),
					None => return Err(Error::new_with_span("integer overflow", span.clone()))
				}
			}
		}
		
		Err(Error::new_with_span("invalid operands", span.clone()))
	}
	

	pub fn perform_rel<F>(op: F, a: &ExpressionValue, b: &ExpressionValue, span: &Span) -> Result<ExpressionValue, Error>
	where F: Fn(&Integer, &Integer) -> bool
	{
		if let Some(ma) = a.as_integer()
		{
			if let Some(mb) = b.as_integer()
				{ return Ok(ExpressionValue::Boolean(op(&ma, &mb))); }
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

	
impl<'p, 'tok> ExpressionParser<'p, 'tok>
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
			let (leftmost_bit, leftmost_span) = try!(self.parser.expect_number());
			try!(self.parser.expect_operator(":"));
			let (rightmost_bit, rightmost_span) = try!(self.parser.expect_number());
			try!(self.parser.expect_operator("]"));
			
			if leftmost_bit < rightmost_bit
				{ return Err(Error::new_with_span("invalid slice", leftmost_span.join(&rightmost_span))); }
			
			if leftmost_bit > 63
				{ return Err(Error::new_with_span("big slice index is currently not supported", leftmost_span.clone())); }
				
			if rightmost_bit > 63
				{ return Err(Error::new_with_span("big slice index is currently not supported", rightmost_span.clone())); }
			
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
			
			match Integer::new_from_str(radix, value_str)
			{
				None => Err(Error::new_with_span("invalid value", value_span)),
				Some(mut integer) =>
				{
					if data_width != 0
					{
						integer = Integer::new_with_explicit_width(integer.value, data_width);
						
						if integer.get_minimum_width() > data_width
							{ return Err(Error::new_with_span("value does not fit given width", value_span)); }
					}
				
					Ok(Expression
					{
						span: value_span,
						term: ExpressionTerm::LiteralInteger(integer)
					})
				}
			}
		}
		
		else
			{ Err(Error::new_with_span("expected expression", self.parser.current().span.clone())) }
	}
}