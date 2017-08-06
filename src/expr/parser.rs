use syntax::{TokenKind, Parser, excerpt_as_usize, excerpt_as_bigint};
use super::{Expression, ExpressionValue, UnaryOp, BinaryOp};


pub struct ExpressionParser<'a, 't>
where 't: 'a
{
	parser: &'a mut Parser<'t>
}


impl Expression
{
	pub fn parse(parser: &mut Parser) -> Result<Expression, ()>
	{
		ExpressionParser::new(parser).parse_expr()
	}
}


impl<'a, 't> ExpressionParser<'a, 't>
{
	pub fn new(parser: &'a mut Parser<'t>) -> ExpressionParser<'a, 't>
	{
		ExpressionParser
		{
			parser: parser
		}
	}
	
	
	pub fn parse_expr(&mut self) -> Result<Expression, ()>
	{
		self.parse_concat()
	}
	
	
	fn parse_unary_ops<F>(&mut self, ops: &[(TokenKind, UnaryOp)], parse_inner: F) -> Result<Expression, ()>
	where F: Fn(&mut ExpressionParser<'a, 't>) -> Result<Expression, ()>
	{
		for op in ops
		{
			let tk = match self.parser.maybe_expect(op.0)
			{
				Some(tk) => tk,
				None => continue
			};
				
			let inner = self.parse_unary_ops(ops, parse_inner)?;
			let span = tk.span.join(&inner.span());
			
			return Ok(Expression::UnaryOp(span, tk.span.clone(), op.1, Box::new(inner)));
		}
		
		parse_inner(self)
	}
	

	fn parse_binary_ops<F>(&mut self, ops: &[(TokenKind, BinaryOp)], parse_inner: F) -> Result<Expression, ()>
	where F: Fn(&mut ExpressionParser<'a, 't>) -> Result<Expression, ()>
	{
		let mut lhs = parse_inner(self)?;
		
		loop
		{
			let mut op_match = None;
			
			for op in ops
			{
				if let Some(tk) = self.parser.maybe_expect(op.0)
				{
					op_match = Some((tk, op.1));
					break;
				}
			}
			
			if let Some(op_match) = op_match
			{				
				let rhs = parse_inner(self)?;
				let span = lhs.span().join(&rhs.span());
				
				lhs = Expression::BinaryOp(span, op_match.0.span.clone(), op_match.1, Box::new(lhs), Box::new(rhs));
			}
			else
				{ break; }
		}
		
		Ok(lhs)
	}
	
	
	fn parse_concat(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::At, BinaryOp::Concat)
			],
			|s| s.parse_lazy_or())
	}
	
	
	fn parse_lazy_or(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::VerticalBarVerticalBar, BinaryOp::LazyOr)
			],
			|s| s.parse_lazy_and())
	}
	
	
	fn parse_lazy_and(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::AmpersandAmpersand, BinaryOp::LazyAnd)
			],
			|s| s.parse_relational())
	}
	
	
	fn parse_relational(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::EqualEqual,       BinaryOp::Eq),
				(TokenKind::ExclamationEqual, BinaryOp::Ne),
				(TokenKind::LessThan,         BinaryOp::Lt),
				(TokenKind::LessThanEqual,    BinaryOp::Le),
				(TokenKind::GreaterThan,      BinaryOp::Gt),
				(TokenKind::GreaterThanEqual, BinaryOp::Ge)
			],
			|s| s.parse_binary_or())
	}
	
	
	fn parse_binary_or(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::VerticalBar, BinaryOp::Or),
			],
			|s| s.parse_binary_xor())
	}
	
	
	fn parse_binary_xor(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::Circumflex, BinaryOp::Xor),
			],
			|s| s.parse_binary_and())
	}
	
	
	fn parse_binary_and(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::Ampersand, BinaryOp::And),
			],
			|s| s.parse_shifts())
	}
	
	
	fn parse_shifts(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::LessThanLessThan,       BinaryOp::Shl),
				(TokenKind::GreaterThanGreaterThan, BinaryOp::Shr)
			],
			|s| s.parse_addition())
	}
	
	
	fn parse_addition(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::Plus,  BinaryOp::Add),
				(TokenKind::Minus, BinaryOp::Sub)
			],
			|s| s.parse_multiplication())
	}
	
	
	fn parse_multiplication(&mut self) -> Result<Expression, ()>
	{
		self.parse_binary_ops(
			&[
				(TokenKind::Asterisk, BinaryOp::Mul),
				(TokenKind::Slash,    BinaryOp::Div),
				(TokenKind::Percent,  BinaryOp::Mod)
			],
			|s| s.parse_bitslice())
	}
	
	
	fn parse_bitslice(&mut self) -> Result<Expression, ()>
	{
		let inner = self.parse_unary()?;
		
		let tk_open = match self.parser.maybe_expect(TokenKind::BracketOpen)
		{
			Some(tk) => tk,
			None => return Ok(inner)
		};
			
		let tk_leftmost = self.parser.expect(TokenKind::Number)?;
		self.parser.expect(TokenKind::Colon)?;
		let tk_rightmost = self.parser.expect(TokenKind::Number)?;
		let tk_close = self.parser.expect(TokenKind::BracketClose)?;
		
		let leftmost  = excerpt_as_usize(self.parser.report, tk_leftmost. excerpt.as_ref().unwrap(), &tk_leftmost .span)?;
		let rightmost = excerpt_as_usize(self.parser.report, tk_rightmost.excerpt.as_ref().unwrap(), &tk_rightmost.span)?;
		
		let slice_span = tk_open.span.join(&tk_close.span);
		let span = inner.span().join(&tk_close.span);
		
		if leftmost < rightmost
			{ return Err(self.parser.report.error_span("invalid bit slice range", &slice_span)); }
			
		Ok(Expression::BitSlice(span, slice_span, leftmost, rightmost, Box::new(inner)))
	}
	
	
	fn parse_unary(&mut self) -> Result<Expression, ()>
	{
		self.parse_unary_ops(
			&[
				(TokenKind::Exclamation, UnaryOp::Not),
				(TokenKind::Minus,       UnaryOp::Neg)
			],
			|s| s.parse_leaf())
	}
	
	
	fn parse_leaf(&mut self) -> Result<Expression, ()>
	{
		if self.parser.next_is(0, TokenKind::ParenOpen)
			{ self.parse_parenthesized() }
	
		else if self.parser.next_is(0, TokenKind::Identifier)
			{ self.parse_variable() }
			
		else if self.parser.next_is(0, TokenKind::Dot)
			{ self.parse_variable() }
			
		else if self.parser.next_is(0, TokenKind::Number)
			{ self.parse_number() }
			
		else
		{
			let span = self.parser.prev().span.after();
			Err(self.parser.report.error_span("expected expression", &span))
		}
	}
	
	
	fn parse_parenthesized(&mut self) -> Result<Expression, ()>
	{
		self.parser.expect(TokenKind::ParenOpen)?;
		let expr = self.parse_expr()?;
		self.parser.expect(TokenKind::ParenClose)?;
		Ok(expr)
	}
	
	
	fn parse_variable(&mut self) -> Result<Expression, ()>
	{
		let tk_dot = self.parser.maybe_expect(TokenKind::Dot);
		let mut name = if tk_dot.is_some() { "." } else { "" }.to_string();
		
		let tk_name = self.parser.expect(TokenKind::Identifier)?;
		name.push_str(&tk_name.excerpt.clone().unwrap());
		
		if let Some(tk_dot) = tk_dot
			{ Ok(Expression::Variable(tk_dot.span.join(&tk_name.span), name)) }
		else
			{ Ok(Expression::Variable(tk_name.span.clone(), name)) }
	}
	
	
	fn parse_number(&mut self) -> Result<Expression, ()>
	{
		let tk_number = self.parser.expect(TokenKind::Number)?;
		let number = tk_number.excerpt.clone().unwrap();
		
		let (bigint, width) = excerpt_as_bigint(self.parser.report, &number, &tk_number.span)?;
		
		let span = tk_number.span;
		let expr = Expression::Literal(span.clone(), ExpressionValue::Integer(bigint));
		
		match width
		{
			None => Ok(expr),
			Some(width) => Ok(Expression::BitSlice(span.clone(), span, width - 1, 0, Box::new(expr)))
		}
	}
}