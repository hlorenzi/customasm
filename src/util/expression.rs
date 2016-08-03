use util::bitvec::BitVec;
use util::error::Error;
use util::label::LabelContext;
use util::parser::Parser;
use util::tokenizer::Span;


pub struct Expression
{
	span: Span,
	term: ExpressionTerm
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


struct ExpressionParser<'p, 'f: 'p, 'tok: 'p>
{
	parser: &'p mut Parser<'f, 'tok>
}


pub struct ExpressionResolver<'owner>
{
	get_global: &'owner Fn(&str) -> Option<&'owner BitVec>,
	get_local: Option<&'owner Fn(&str) -> Option<&'owner BitVec>>
}


impl Expression
{
	pub fn new_by_parsing<'p, 'f: 'p, 'tok: 'p>(parser: &'p mut Parser<'f, 'tok>) -> Result<Expression, Error>
	{
		let mut expr_parser = ExpressionParser { parser: parser };
		expr_parser.parse()
	}
	
	
	pub fn can_resolve(&self, resolver: &ExpressionResolver) -> bool
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(_)       => true,
			&ExpressionTerm::GlobalVariable(ref name) => (resolver.get_global)(&name).is_some(),
			&ExpressionTerm::LocalVariable(ref name)  =>
				if resolver.get_local.is_none()
					{ false }
				else
					{ resolver.get_local.unwrap()(&name).is_some() },
			
			&ExpressionTerm::Add(ref lhs, ref rhs)        => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::Subtract(ref lhs, ref rhs)   => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::Multiply(ref lhs, ref rhs)   => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::Divide(ref lhs, ref rhs)     => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::ShiftLeft(ref lhs, ref rhs)  => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::ShiftRight(ref lhs, ref rhs) => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::And(ref lhs, ref rhs)        => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::Or(ref lhs, ref rhs)         => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			&ExpressionTerm::Xor(ref lhs, ref rhs)        => lhs.can_resolve(resolver) && rhs.can_resolve(resolver),
			
			&ExpressionTerm::Slice(ref expr, _, _) => expr.can_resolve(resolver)
		}
	}
	
	
	pub fn get_minimum_bit_num(&self, resolver: &ExpressionResolver, undeclared_size: usize) -> usize
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(ref bitvec) => bitvec.len(),
			
			&ExpressionTerm::GlobalVariable(ref name) =>
				match (resolver.get_global)(&name)
				{
					Some(bitvec) => bitvec.len(),
					None => undeclared_size
				},
				
			&ExpressionTerm::LocalVariable(ref name) =>
				if resolver.get_local.is_none()
					{ undeclared_size }
				else
				{
					match resolver.get_local.unwrap()(&name)
					{
						Some(bitvec) => bitvec.len(),
						None => undeclared_size
					}
				},
			
			&ExpressionTerm::Slice(ref expr, left, right) =>
				if left > right
					{ left - right + 1 }
				else
					{ right - left + 1 },
					
			_ => undeclared_size
		}
	}
	
	
	pub fn resolve(&self, resolver: &ExpressionResolver) -> Result<BitVec, Error>
	{
		match &self.term
		{
			&ExpressionTerm::LiteralUInt(ref bitvec)  => Ok(bitvec.clone()),
			
			&ExpressionTerm::GlobalVariable(ref name) =>
				match (resolver.get_global)(&name)
				{
					Some(bitvec) => Ok(bitvec.clone()),
					None => Err(Error::new_with_span(format!("undeclared `{}`", name), self.span.clone()))
				},
				
			&ExpressionTerm::LocalVariable(ref name) =>
				if resolver.get_local.is_none()
					{ Err(Error::new_with_span(format!("undeclared local `{}`", name), self.span.clone())) }
				else
				{
					match resolver.get_local.unwrap()(&name)
					{
						Some(bitvec) => Ok(bitvec.clone()),
						None => Err(Error::new_with_span(format!("undeclared local `{}`", name), self.span.clone()))
					}
				},
			
			&ExpressionTerm::Add(ref lhs, _)        => lhs.resolve(resolver),
			&ExpressionTerm::Subtract(ref lhs, _)   => lhs.resolve(resolver),
			&ExpressionTerm::Multiply(ref lhs, _)   => lhs.resolve(resolver),
			&ExpressionTerm::Divide(ref lhs, _)     => lhs.resolve(resolver),
			&ExpressionTerm::ShiftLeft(ref lhs, _)  => lhs.resolve(resolver),
			&ExpressionTerm::ShiftRight(ref lhs, _) => lhs.resolve(resolver),
			&ExpressionTerm::And(ref lhs, _)        => lhs.resolve(resolver),
			&ExpressionTerm::Or(ref lhs, _)         => lhs.resolve(resolver),
			&ExpressionTerm::Xor(ref lhs, _)        => lhs.resolve(resolver),
			
			&ExpressionTerm::Slice(ref expr, left, right) => Ok(try!(expr.resolve(resolver)).slice(left, right))
		}
	}
}


impl<'owner> ExpressionResolver<'owner>
{
	pub fn new(
		get_global: &'owner Fn(&str) -> Option<&'owner BitVec>,
		get_local: &'owner Fn(&str) -> Option<&'owner BitVec>) -> ExpressionResolver<'owner>
	{
		ExpressionResolver
		{
			get_global: get_global,
			get_local: Some(get_local)
		}
	}
	
	
	pub fn new_without_locals(get_global: &'owner Fn(&str) -> Option<&'owner BitVec>) -> ExpressionResolver<'owner>
	{
		ExpressionResolver
		{
			get_global: get_global,
			get_local: None
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
		let mut lhs = try!(self.parse_leaf_term());
		
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
			
			let rhs = try!(self.parse_leaf_term());
			
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
	
	
	fn parse_leaf_term(&mut self) -> Result<Expression, Error>
	{
		if self.parser.current().is_identifier()
		{
			Ok(Expression
			{
				span: self.parser.current().span.clone(),
				term: ExpressionTerm::GlobalVariable(try!(self.parser.expect_identifier()).identifier().clone())
			})
		}
		
		else if self.parser.current().is_operator("'") && self.parser.next(1).is_identifier()
		{
			let start_span = self.parser.current().span.clone();
			self.parser.advance();
			Ok(Expression
			{
				span: self.parser.current().span.join(&start_span),
				term: ExpressionTerm::LocalVariable(try!(self.parser.expect_identifier()).identifier().clone())
			})
		}
		
		else if self.parser.current().is_number()
		{
			let token = try!(self.parser.expect_number()).clone();
			let (radix, value_str) = token.number();
			
			match BitVec::new_from_str_trimmed(radix, value_str)
			{
				Err(msg) => Err(self.parser.make_error(msg, &token.span)),
				Ok(bitvec) => 
					Ok(Expression
					{
						span: token.span.clone(),
						term: ExpressionTerm::LiteralUInt(bitvec)
					})
			}
		}
		
		else
			{ Err(self.parser.make_error("expected expression", &self.parser.current().span)) }
	}
}