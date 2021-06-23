use crate::*;


pub struct ExpressionParser<'a, 'parser: 'a>
{
	parser: &'a mut syntax::Parser<'parser>,
}


impl expr::Expr
{
	pub fn parse(parser: &mut syntax::Parser) -> Result<expr::Expr, ()>
	{
		ExpressionParser::new(parser).parse_expr()
	}
}


impl<'a, 'parser> ExpressionParser<'a, 'parser>
{
	pub fn new(parser: &'a mut syntax::Parser<'parser>) -> ExpressionParser<'a, 'parser>
	{
		ExpressionParser
		{
			parser,
		}
	}
	
	
	pub fn parse_expr(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_ternary_conditional()
	}
	
	
	fn parse_unary_ops<F>(&mut self, ops: &[(syntax::TokenKind, expr::UnaryOp)], parse_inner: F) -> Result<expr::Expr, ()>
	where F: Fn(&mut ExpressionParser<'a, 'parser>) -> Result<expr::Expr, ()>
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
			
			return Ok(expr::Expr::UnaryOp(span, tk.span.clone(), op.1, Box::new(inner)));
		}
		
		parse_inner(self)
	}
	

	fn parse_binary_ops<F>(&mut self, ops: &[(syntax::TokenKind, expr::BinaryOp)], parse_inner: F) -> Result<expr::Expr, ()>
	where F: Fn(&mut ExpressionParser<'a, 'parser>) -> Result<expr::Expr, ()>
	{
		let mut lhs = parse_inner(self)?;
		
		loop
		{
			if self.parser.next_is_linebreak()
				{ break; }

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
				
				lhs = expr::Expr::BinaryOp(span, op_match.0.span.clone(), op_match.1, Box::new(lhs), Box::new(rhs));
			}
			else
				{ break; }
		}
		
		Ok(lhs)
	}
	

	fn parse_right_associative_binary_ops<F>(&mut self, ops: &[(syntax::TokenKind, expr::BinaryOp)], parse_inner: F) -> Result<expr::Expr, ()>
	where F: Fn(&mut ExpressionParser<'a, 'parser>) -> Result<expr::Expr, ()>
	{
		let mut lhs = parse_inner(self)?;
		
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
			let rhs = self.parse_expr()?;
			let span = lhs.span().join(&rhs.span());
			
			lhs = expr::Expr::BinaryOp(span, op_match.0.span.clone(), op_match.1, Box::new(lhs), Box::new(rhs));
		}
		
		Ok(lhs)
	}
	
	
	fn parse_ternary_conditional(&mut self) -> Result<expr::Expr, ()>
	{
		let cond = self.parse_assignment()?;
		
		if self.parser.maybe_expect(syntax::TokenKind::Question).is_some()
		{
			let true_branch = self.parse_expr()?;
			
			let false_branch =
			{
				if self.parser.maybe_expect(syntax::TokenKind::Colon).is_some()
					{ self.parse_assignment()? }
				else
					{ expr::Expr::Block(true_branch.span(), Vec::new()) }
			};
			
			Ok(expr::Expr::TernaryOp(cond.span().join(&false_branch.span()), Box::new(cond), Box::new(true_branch), Box::new(false_branch)))
		}
		else
			{ Ok(cond) }
	}
	
	
	fn parse_assignment(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_right_associative_binary_ops(
			&[
				(syntax::TokenKind::Equal, expr::BinaryOp::Assign)
			],
			|s| s.parse_concat())
	}
	
	
	fn parse_concat(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::At, expr::BinaryOp::Concat)
			],
			|s| s.parse_lazy_or())
	}
	
	
	fn parse_lazy_or(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::VerticalBarVerticalBar, expr::BinaryOp::LazyOr)
			],
			|s| s.parse_lazy_and())
	}
	
	
	fn parse_lazy_and(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::AmpersandAmpersand, expr::BinaryOp::LazyAnd)
			],
			|s| s.parse_relational())
	}
	
	
	fn parse_relational(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::EqualEqual,       expr::BinaryOp::Eq),
				(syntax::TokenKind::ExclamationEqual, expr::BinaryOp::Ne),
				(syntax::TokenKind::LessThan,         expr::BinaryOp::Lt),
				(syntax::TokenKind::LessThanEqual,    expr::BinaryOp::Le),
				(syntax::TokenKind::GreaterThan,      expr::BinaryOp::Gt),
				(syntax::TokenKind::GreaterThanEqual, expr::BinaryOp::Ge)
			],
			|s| s.parse_binary_or())
	}
	
	
	fn parse_binary_or(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::VerticalBar, expr::BinaryOp::Or),
			],
			|s| s.parse_binary_xor())
	}
	
	
	fn parse_binary_xor(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::Circumflex, expr::BinaryOp::Xor),
			],
			|s| s.parse_binary_and())
	}
	
	
	fn parse_binary_and(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::Ampersand, expr::BinaryOp::And),
			],
			|s| s.parse_shifts())
	}
	
	
	fn parse_shifts(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::LessThanLessThan,       expr::BinaryOp::Shl),
				(syntax::TokenKind::GreaterThanGreaterThan, expr::BinaryOp::Shr)
			],
			|s| s.parse_addition())
	}
	
	
	fn parse_addition(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::Plus,  expr::BinaryOp::Add),
				(syntax::TokenKind::Minus, expr::BinaryOp::Sub)
			],
			|s| s.parse_multiplication())
	}
	
	
	fn parse_multiplication(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::Asterisk, expr::BinaryOp::Mul),
				(syntax::TokenKind::Slash,    expr::BinaryOp::Div),
				(syntax::TokenKind::Percent,  expr::BinaryOp::Mod)
			],
			|s| s.parse_bitslice())
	}
	
	
	fn parse_bitslice(&mut self) -> Result<expr::Expr, ()>
	{
		let inner = self.parse_size()?;
		
		if self.parser.next_is_linebreak()
			{ return Ok(inner); }

		let tk_open = match self.parser.maybe_expect(syntax::TokenKind::BracketOpen)
		{
			Some(tk) => tk,
			None => return Ok(inner)
		};
			
		let tk_leftmost = self.parser.expect(syntax::TokenKind::Number)?;
		self.parser.expect(syntax::TokenKind::Colon)?;
		let tk_rightmost = self.parser.expect(syntax::TokenKind::Number)?;
		let tk_close = self.parser.expect(syntax::TokenKind::BracketClose)?;
		
		let leftmost  = syntax::excerpt_as_usize(self.parser.report.clone(), tk_leftmost. excerpt.as_ref().unwrap(), &tk_leftmost .span)?;
		let rightmost = syntax::excerpt_as_usize(self.parser.report.clone(), tk_rightmost.excerpt.as_ref().unwrap(), &tk_rightmost.span)?;
		
		let slice_span = tk_open.span.join(&tk_close.span);
		let span = inner.span().join(&tk_close.span);
		
		if leftmost < rightmost
		{
			if let Some(ref report) = self.parser.report
			{
				report.error_span("invalid bit slice range", &slice_span);
			}
			return Err(());
		}
			
		Ok(expr::Expr::BitSlice(span, slice_span, leftmost + 1, rightmost, Box::new(inner)))
	}
	
	
	fn parse_size(&mut self) -> Result<expr::Expr, ()>
	{
		let inner = self.parse_unary()?;
		
		if self.parser.next_is_linebreak()
			{ return Ok(inner); }
		
		let tk_grave = match self.parser.maybe_expect(syntax::TokenKind::Grave)
		{
			Some(tk) => tk,
			None => return Ok(inner)
		};

		let tk_size = self.parser.expect(syntax::TokenKind::Number)?;
		let size = syntax::excerpt_as_usize(
			self.parser.report.clone(),
			tk_size.excerpt.as_ref().unwrap(),
			&tk_size.span)?;

		let span = inner.span().join(&tk_size.span);
		let size_span = tk_grave.span.join(&tk_size.span);

		Ok(expr::Expr::BitSlice(span, size_span, size, 0, Box::new(inner)))
	}
	
	
	fn parse_unary(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_unary_ops(
			&[
				(syntax::TokenKind::Exclamation, expr::UnaryOp::Not),
				(syntax::TokenKind::Minus,       expr::UnaryOp::Neg)
			],
			|s| s.parse_call())
	}
	
	
	fn parse_call(&mut self) -> Result<expr::Expr, ()>
	{
		let leaf = self.parse_leaf()?;
		
		if self.parser.next_is_linebreak()
			{ return Ok(leaf); }
			
		if self.parser.maybe_expect(syntax::TokenKind::ParenOpen).is_none()
			{ return Ok(leaf); }
			
		let mut args = Vec::new();
		while !self.parser.next_is(0, syntax::TokenKind::ParenClose)
		{
			args.push(self.parse_expr()?);
			
			if self.parser.next_is(0, syntax::TokenKind::ParenClose)
				{ break; }
				
			self.parser.expect(syntax::TokenKind::Comma)?;
		}
		
		let tk_close = self.parser.expect(syntax::TokenKind::ParenClose)?;
		
		Ok(expr::Expr::Call(leaf.span().join(&tk_close.span), Box::new(leaf), args))
	}
	
	
	fn parse_leaf(&mut self) -> Result<expr::Expr, ()>
	{
		if self.parser.next_is(0, syntax::TokenKind::BraceOpen)
			{ self.parse_block() }
	
		else if self.parser.next_is(0, syntax::TokenKind::ParenOpen)
			{ self.parse_parenthesized() }
	
		else if self.parser.next_is(0, syntax::TokenKind::Identifier)
			{ self.parse_variable() }
			
		else if self.parser.next_is(0, syntax::TokenKind::Dot)
			{ self.parse_variable() }
			
		else if self.parser.next_is(0, syntax::TokenKind::Number)
			{ self.parse_number() }
			
		else if self.parser.next_is(0, syntax::TokenKind::String)
			{ self.parse_string() }
	
		else if self.parser.next_is(0, syntax::TokenKind::KeywordAsm)
			{ self.parse_asm() }
			
		else
		{
			if let Some(ref report) = self.parser.report
			{
				let span = self.parser.get_span_after_prev();
				report.error_span("expected expression", &span);
			}
			Err(())
		}
	}
	
	
	fn parse_block(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_open = self.parser.expect(syntax::TokenKind::BraceOpen)?;
		
		let mut exprs = Vec::new();
		while !self.parser.next_is(0, syntax::TokenKind::BraceClose)
		{
			exprs.push(self.parse_expr()?);
			
			if self.parser.maybe_expect_linebreak().is_some()
				{ continue; }
				
			if self.parser.next_is(0, syntax::TokenKind::BraceClose)
				{ break; }
				
			self.parser.expect(syntax::TokenKind::Comma)?;
		}
		
		let tk_close = self.parser.expect(syntax::TokenKind::BraceClose)?;
		
		Ok(expr::Expr::Block(tk_open.span.join(&tk_close.span), exprs))
	}
	
	
	fn parse_parenthesized(&mut self) -> Result<expr::Expr, ()>
	{
		self.parser.expect(syntax::TokenKind::ParenOpen)?;
		let expr = self.parse_expr()?;
		self.parser.expect(syntax::TokenKind::ParenClose)?;
		Ok(expr)
	}
	
	
	fn parse_variable(&mut self) -> Result<expr::Expr, ()>
	{
		let mut span = diagn::Span::new_dummy();
		let mut hierarchy_level = 0;
		
		while let Some(tk_dot) = self.parser.maybe_expect(syntax::TokenKind::Dot)
		{
			hierarchy_level += 1;
			span = span.join(&tk_dot.span);
		}

		let mut hierarchy = Vec::new();

		loop
		{
			let tk_name = self.parser.expect(syntax::TokenKind::Identifier)?;
			let name = tk_name.excerpt.clone().unwrap();
			hierarchy.push(name);
			span = span.join(&tk_name.span);

			if self.parser.maybe_expect(syntax::TokenKind::Dot).is_none()
			{
				break;
			}
		}
		
		let expr_var = expr::Expr::Variable(span, hierarchy_level, hierarchy);

		/*if let Some(rule_params) = self.rule_params
		{
			if let Some(rule_param) = rule_params.iter().find(|p| p.name == name)
			{
				let width = match rule_param.typ
				{
					RuleParameterType::Unsigned(w) => Some(w),
					RuleParameterType::Signed(w) => Some(w),
					RuleParameterType::Integer(w) => Some(w),
					_ => None
				};

				if let Some(width) = width
				{
					if width > 0
						{ return Ok(expr::Expr::SoftSlice(expr_span.clone(), expr_span.clone(), width - 1, 0, Box::new(expr_var))); }
				}
			}
		}*/

		Ok(expr_var)
	}
	
	
	fn parse_number(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_number = self.parser.expect(syntax::TokenKind::Number)?;
		let number = tk_number.excerpt.clone().unwrap();
		
		let bigint = syntax::excerpt_as_bigint(self.parser.report.clone(), &number, &tk_number.span)?;
		
		let span = tk_number.span;
		let expr = expr::Expr::Literal(
			span.clone(),
			expr::Value::Integer(bigint));

		Ok(expr)
	}
	
	
	fn parse_string(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_str = self.parser.expect(syntax::TokenKind::String)?;

		let string = syntax::excerpt_as_string_contents(
			self.parser.report.clone().unwrap_or(diagn::RcReport::new()),
			tk_str.excerpt.as_ref().unwrap(),
			&tk_str.span)?;
		
		let expr = expr::Expr::Literal(
			tk_str.span.clone(),
			expr::Value::Integer(util::BigInt::new_from_str(&string)));

		Ok(expr)
	}
	
	
	fn parse_asm(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_asm = self.parser.expect(syntax::TokenKind::KeywordAsm)?;
		self.parser.expect(syntax::TokenKind::BraceOpen)?;

		let contents = self.parser.slice_until_token_over_nested_braces(syntax::TokenKind::BraceClose);

		let tk_brace_close = self.parser.expect(syntax::TokenKind::BraceClose)?;

		let expr = expr::Expr::Asm(
			tk_asm.span.clone().join(&tk_brace_close.span),
			contents.get_cloned_tokens());

		Ok(expr)
	}
}