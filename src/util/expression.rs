use util::bigint::BigInt;
use util::error::Error;
use util::parser::Parser;
use util::tokenizer::Span;


pub struct Expression
{
	pub span: Span,
	pub term: ExpressionTerm
}


#[derive(Clone)]
pub enum ExpressionValue
{
	Integer(BigInt),
	Boolean(bool)
}


pub enum ExpressionVariable
{
	Global(String),
	Local(String)
}


pub enum ExpressionTerm
{
	Literal(ExpressionValue),
	Variable(ExpressionVariable),
	Binary(Box<Expression>, Box<Expression>, BinaryOp),	
	Slice(Box<Expression>, usize, usize)
}


#[derive(Copy, Clone)]
pub enum BinaryOp
{
	Add, Sub, Mul, Div,
	Shl, Shr,
	BitAnd, BitOr, BitXor,
	
	And, Or,
	
	Eq, Ne,
	Lt, Gt,
	Le, Ge
}


struct ExpressionParser<'p, 'tok: 'p>
{
	parser: &'p mut Parser<'tok>,
	check_var: Option<&'p Fn(&ExpressionVariable, &Span) -> Result<(), Error>>
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
		check_var: &'p Fn(&ExpressionVariable, &Span) -> Result<(), Error>)
		-> Result<Expression, Error>
	{
		let mut expr_parser = ExpressionParser { parser: parser, check_var: Some(check_var) };
		expr_parser.parse()
	}
	
	
	pub fn new_literal_integer(value: i64, span: Span) -> Expression
	{
		Expression
		{
			span: span,
			term: ExpressionTerm::Literal(ExpressionValue::Integer(BigInt::from_i64(value)))
		}
	}
	
	
	pub fn can_resolve<F>(&self, handle_var: &F) -> Result<bool, Error>
	where F: Fn(&ExpressionVariable, &Span) -> Result<bool, Error>
	{
		match &self.term
		{
			&ExpressionTerm::Literal(_)      => Ok(true),
			&ExpressionTerm::Variable(ref v) => handle_var(&v, &self.span),
			
			&ExpressionTerm::Binary(ref lhs, ref rhs, _) =>
				Ok(try!(lhs.can_resolve(handle_var)) && try!(rhs.can_resolve(handle_var))),
				
			&ExpressionTerm::Slice(ref expr, _, _) => expr.can_resolve(handle_var)
		}
	}
	
	
	pub fn get_explicit_bit_num(&self) -> Option<usize>
	{
		match &self.term
		{
			&ExpressionTerm::Slice(_, left, right) => Some(left - right + 1),
					
			_ => None
		}
	}
	
	
	pub fn get_minimum_bit_num<F>(&self, handle_var: &F) -> Result<Option<usize>, Error>
	where F: Fn(&ExpressionVariable, &Span) -> Result<Option<usize>, Error>
	{
		match &self.term
		{
			&ExpressionTerm::Literal(ref v) => match v
			{
				&ExpressionValue::Integer(ref bigint) => Ok(Some(bigint.width())),
				_ => Ok(None)
			},
			
			&ExpressionTerm::Variable(ref v) => handle_var(&v, &self.span),				
			
			&ExpressionTerm::Slice(_, left, right) => Ok(Some(left - right + 1)),
					
			_ => Ok(None)
		}
	}
	
	
	pub fn resolve<F>(&self, handle_var: &F) -> Result<ExpressionValue, Error>
	where F: Fn(&ExpressionVariable, &Span) -> Result<ExpressionValue, Error>
	{
		match &self.term
		{
			&ExpressionTerm::Literal(ref v)  => Ok(v.clone()),
			&ExpressionTerm::Variable(ref v) => handle_var(&v, &self.span),			
			
			&ExpressionTerm::Binary(ref lhs, ref rhs, op) =>
			{
				let lhs_value = try!(lhs.resolve(handle_var));
				let rhs_value = try!(rhs.resolve(handle_var));
				
				match op
				{
					BinaryOp::Add => ExpressionValue::perform_binary_arith(|a, b| a.checked_add(b),  &lhs_value, &rhs_value, &self.span),
					BinaryOp::Sub => ExpressionValue::perform_binary_arith(|a, b| a.checked_sub(b),  &lhs_value, &rhs_value, &self.span),
					BinaryOp::Mul => ExpressionValue::perform_binary_arith(|a, b| a.checked_mul(b),  &lhs_value, &rhs_value, &self.span),
					BinaryOp::Div => ExpressionValue::perform_binary_arith(|a, b| a.checked_div(b),  &lhs_value, &rhs_value, &self.span),
					BinaryOp::Shl => ExpressionValue::perform_binary_arith(|a, b| a.checked_shl(b), &lhs_value, &rhs_value, &self.span),
					BinaryOp::Shr => ExpressionValue::perform_binary_arith(|a, b| a.checked_shr(b), &lhs_value, &rhs_value, &self.span),
					
					BinaryOp::BitAnd => ExpressionValue::perform_binary_arith(|a, b| a.bit_and(b), &lhs_value, &rhs_value, &self.span),
					BinaryOp::BitOr  => ExpressionValue::perform_binary_arith(|a, b| a.bit_or(b),  &lhs_value, &rhs_value, &self.span),
					BinaryOp::BitXor => ExpressionValue::perform_binary_arith(|a, b| a.bit_xor(b), &lhs_value, &rhs_value, &self.span),
					
					BinaryOp::And => ExpressionValue::perform_bool(|a, b| a & b, &lhs_value, &rhs_value, &self.span),
					BinaryOp::Or  => ExpressionValue::perform_bool(|a, b| a | b, &lhs_value, &rhs_value, &self.span),
					
					BinaryOp::Eq => ExpressionValue::perform_rel(|a, b| a.eq(b), &lhs_value, &rhs_value, &self.span),
					BinaryOp::Ne => ExpressionValue::perform_rel(|a, b| a.ne(b), &lhs_value, &rhs_value, &self.span),
					BinaryOp::Lt => ExpressionValue::perform_rel(|a, b| a.lt(b), &lhs_value, &rhs_value, &self.span),
					BinaryOp::Gt => ExpressionValue::perform_rel(|a, b| a.gt(b), &lhs_value, &rhs_value, &self.span),
					BinaryOp::Le => ExpressionValue::perform_rel(|a, b| a.le(b), &lhs_value, &rhs_value, &self.span),
					BinaryOp::Ge => ExpressionValue::perform_rel(|a, b| a.ge(b), &lhs_value, &rhs_value, &self.span),
				}
			}
			
			&ExpressionTerm::Slice(ref expr, left, right) => ExpressionValue::perform_unary_arith(
				|a| a.slice(left, right), &try!(expr.resolve(handle_var)), &self.span)
		}
	}
}


impl ExpressionValue
{
	pub fn as_integer(&self) -> Option<BigInt>
	{
		match self
		{
			&ExpressionValue::Integer(ref bigint) => Some(bigint.clone()),
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
	where F: Fn(&BigInt) -> Option<BigInt>
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
	where F: Fn(&BigInt, &BigInt) -> Option<BigInt>
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
	where F: Fn(&BigInt, &BigInt) -> bool
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
		static BINARY_OPS: &'static [&'static [(&'static str, BinaryOp)]] =
		&[
			&[("||", BinaryOp::Or)],
			&[("&&", BinaryOp::And)],
			&[
				("==", BinaryOp::Eq),
				("!=", BinaryOp::Ne),
				("<",  BinaryOp::Lt),
				(">",  BinaryOp::Gt),
				("<=", BinaryOp::Le),
				(">=", BinaryOp::Ge)
			],
			&[("|",  BinaryOp::BitOr)],
			&[("^",  BinaryOp::BitXor)],
			&[("&",  BinaryOp::BitAnd)],
			&[
				("<<", BinaryOp::Shl),
				(">>", BinaryOp::Shr)
			],
			&[
				("+",  BinaryOp::Add),
				("-",  BinaryOp::Sub)
			],
			&[
				("*",  BinaryOp::Mul),
				("/",  BinaryOp::Div)
			],
		];
		
		if level >= BINARY_OPS.len()
			{ return self.parse_slice_term(); }
	
		let mut lhs = try!(self.parse_binary_op_term(level + 1));
		
		loop
		{
			let mut op_match = None;
			for op in BINARY_OPS[level]
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
			let term = ExpressionTerm::Binary(Box::new(lhs), Box::new(rhs), op_match.unwrap().1);
			
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
			
			let var = ExpressionVariable::Global(ident);
			
			if let Some(check_var) = self.check_var
				{ try!(check_var(&var, &ident_span)); }
			
			Ok(Expression
			{
				span: ident_span,
				term: ExpressionTerm::Variable(var)
			})
		}
		
		else if self.parser.current().is_operator("'") && self.parser.next(1).is_identifier()
		{
			let start_span = self.parser.current().span.clone();
			self.parser.advance();
			
			let (ident, ident_span) = try!(self.parser.expect_identifier());
			
			let var = ExpressionVariable::Local(ident);
			
			if let Some(check_var) = self.check_var
				{ try!(check_var(&var, &ident_span)); }
			
			Ok(Expression
			{
				span: ident_span.join(&start_span),
				term: ExpressionTerm::Variable(var)
			})
		}
		
		else if self.parser.current().is_number()
		{
			let maybe_data_width = 
				if self.parser.next(1).is_operator("'")
				{
					let (data_width, _) = try!(self.parser.expect_number());
					try!(self.parser.expect_operator("'"));
					Some(data_width)
				}
				else
					{ None };
					
			let (radix, value_str, value_span) = try!(self.parser.expect_number_str());
			
			match BigInt::from_str_radix(radix, value_str)
			{
				None => Err(Error::new_with_span("invalid value", value_span)),
				Some(bigint) =>
				{
					let integer_width = bigint.width();
					
					let literal_expr = Expression
					{
						span: value_span.clone(),
						term: ExpressionTerm::Literal(ExpressionValue::Integer(bigint))
					};
					
					match maybe_data_width
					{
						None => Ok(literal_expr),
						
						Some(data_width) =>
						{
							if integer_width > data_width
								{ return Err(Error::new_with_span("value does not fit given width", value_span)); }
							
							Ok(Expression
							{
								span: value_span.clone(),
								term: ExpressionTerm::Slice(Box::new(literal_expr), data_width - 1, 0)
							})
						}
					}
				}
			}
		}
		
		else
			{ Err(Error::new_with_span("expected expression", self.parser.current().span.clone())) }
	}
}