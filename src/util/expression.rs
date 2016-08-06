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
	Subtract(Box<Expression>, Box<Expression>),
	Multiply(Box<Expression>, Box<Expression>),
	Divide(Box<Expression>, Box<Expression>),
	ShiftLeft(Box<Expression>, Box<Expression>),
	ShiftRight(Box<Expression>, Box<Expression>),
	And(Box<Expression>, Box<Expression>),
	Or(Box<Expression>, Box<Expression>),
	Xor(Box<Expression>, Box<Expression>),
	Slice(Box<Expression>, usize, usize)
}


pub enum ExpressionName<'s>
{
	GlobalVariable(&'s str),
	LocalVariable(&'s str)
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
			
			&ExpressionTerm::Add(ref lhs, ref rhs)        => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Subtract(ref lhs, ref rhs)   => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Multiply(ref lhs, ref rhs)   => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Divide(ref lhs, ref rhs)     => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::ShiftLeft(ref lhs, ref rhs)  => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::ShiftRight(ref lhs, ref rhs) => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::And(ref lhs, ref rhs)        => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Or(ref lhs, ref rhs)         => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			&ExpressionTerm::Xor(ref lhs, ref rhs)        => Ok(try!(lhs.can_resolve(check_name)) && try!(rhs.can_resolve(check_name))),
			
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
	
	
	pub fn get_minimum_bit_num<F>(&self, check_name: &F) -> Result<usize, Error>
	where F: Fn(ExpressionName, &Span) -> Result<usize, Error>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(ref bitvec)  => Ok(bitvec.len()),
			&ExpressionTerm::GlobalVariable(ref name) => check_name(ExpressionName::GlobalVariable(&name), &self.span),				
			&ExpressionTerm::LocalVariable(ref name)  => check_name(ExpressionName::LocalVariable(&name), &self.span),
			
			&ExpressionTerm::Slice(_, left, right) =>
				if left > right
					{ Ok(left - right + 1) }
				else
					{ Ok(right - left + 1) },
					
			_ => Err(Error::new_with_span("unimplemented", self.span.clone()))
		}
	}
	
	
	pub fn resolve<F>(&self, check_name: &F) -> Result<BitVec, Error>
	where F: Fn(ExpressionName, &Span) -> Result<BitVec, Error>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(ref bitvec)  => Ok(bitvec.clone()),
			&ExpressionTerm::GlobalVariable(ref name) => check_name(ExpressionName::GlobalVariable(&name), &self.span),				
			&ExpressionTerm::LocalVariable(ref name)  => check_name(ExpressionName::LocalVariable(&name), &self.span),
			
			&ExpressionTerm::Add(ref lhs, _)        => lhs.resolve(check_name),
			&ExpressionTerm::Subtract(ref lhs, _)   => lhs.resolve(check_name),
			&ExpressionTerm::Multiply(ref lhs, _)   => lhs.resolve(check_name),
			&ExpressionTerm::Divide(ref lhs, _)     => lhs.resolve(check_name),
			&ExpressionTerm::ShiftLeft(ref lhs, _)  => lhs.resolve(check_name),
			&ExpressionTerm::ShiftRight(ref lhs, _) => lhs.resolve(check_name),
			&ExpressionTerm::And(ref lhs, _)        => lhs.resolve(check_name),
			&ExpressionTerm::Or(ref lhs, _)         => lhs.resolve(check_name),
			&ExpressionTerm::Xor(ref lhs, _)        => lhs.resolve(check_name),
			
			&ExpressionTerm::Slice(ref expr, left, right) => Ok(try!(expr.resolve(check_name)).slice(left, right))
		}
	}
}

	
impl<'p, 'f, 'tok> ExpressionParser<'p, 'f, 'tok>
{
	pub fn parse(&mut self) -> Result<Expression, Error>
	{
		self.parse_addition_term()
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
					{ ExpressionTerm::Subtract(Box::new(lhs), Box::new(rhs)) };
			
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
		let mut lhs = try!(self.parse_slice_term());
		
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
			
			let rhs = try!(self.parse_slice_term());
			
			let span = lhs.span.join(&rhs.span);
			
			let term =
				if is_mul
					{ ExpressionTerm::Multiply(Box::new(lhs), Box::new(rhs)) }
				else
					{ ExpressionTerm::Divide(Box::new(lhs), Box::new(rhs)) };
			
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
		if self.parser.current().is_identifier()
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