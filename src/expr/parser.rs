use crate::*;


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker)
    -> Result<expr::Expr, ()>
{
    ExpressionParser::new(report, walker)
        .parse_expr()
}


pub fn parse_optional(
    walker: &mut syntax::Walker)
    -> Option<expr::Expr>
{
    let mut dummy_report = diagn::Report::new();
	parse(&mut dummy_report, walker).ok()
}


struct ExpressionParser<'a, 'src: 'a>
{
    report: &'a mut diagn::Report,
	walker: &'a mut syntax::Walker<'src>,
	recursion_depth: usize,
}


impl<'a, 'src> ExpressionParser<'a, 'src>
{
	pub fn new(
        report: &'a mut diagn::Report,
        walker: &'a mut syntax::Walker<'src>)
        -> ExpressionParser<'a, 'src>
	{
		ExpressionParser {
            report,
			walker,
			recursion_depth: 0,
		}
	}


	pub fn check_recursion_limit(&mut self) -> Result<(), ()>
	{
		if self.recursion_depth > expr::PARSE_RECURSION_DEPTH_MAX
		{
			self.report.message_with_parents_dedup(
				diagn::Message::error_span(
					"expression recursion depth limit reached",
					self.walker.get_cursor_span()));
	
			return Err(());
		}

		Ok(())
	}
	
	
	pub fn parse_expr(&mut self) -> Result<expr::Expr, ()>
	{
		self.recursion_depth += 1;

		self.check_recursion_limit()?;
		
		let maybe_expr = self.parse_ternary_conditional();

		self.recursion_depth -= 1;

		maybe_expr
	}
	
	
	fn parse_unary_ops<F>(
		&mut self,
		ops: &[(syntax::TokenKind, expr::UnaryOp)],
		parse_inner: F)
		-> Result<expr::Expr, ()>
		where F: Fn(&mut ExpressionParser<'a, 'src>) -> Result<expr::Expr, ()>
	{
		for op in ops
		{
			let tk_span = {
				match self.walker.maybe_expect(op.0)
				{
					Some(tk) => tk.span,
					None => continue,
				}
			};
			
			self.recursion_depth += 1;
			self.check_recursion_limit()?;
			
			let inner = self.parse_unary_ops(ops, parse_inner)?;
			let span = tk_span.join(inner.span());
			
			self.recursion_depth -= 1;
			
			return Ok(expr::Expr::UnaryOp(
				span,
				tk_span,
				op.1,
				Box::new(inner)));
		}
		
		parse_inner(self)
	}
	

	fn parse_binary_ops<F>(
		&mut self,
		ops: &[(syntax::TokenKind, expr::BinaryOp)],
		parse_inner: F)
		-> Result<expr::Expr, ()>
		where F: Fn(&mut ExpressionParser<'a, 'src>) -> Result<expr::Expr, ()>
	{
		let mut lhs = parse_inner(self)?;
		
		loop
		{
			if self.walker.next_linebreak().is_some()
				{ break; }

			let mut op_match = None;
			
			for op in ops
			{
				if let Some(tk) = self.walker.maybe_expect(op.0)
				{
					op_match = Some((tk.span, op.1));
					break;
				}
			}
			
			if let Some(op_match) = op_match
			{				
				let rhs = parse_inner(self)?;
				let span = lhs.span().join(rhs.span());
				
				lhs = expr::Expr::BinaryOp(
					span,
					op_match.0,
					op_match.1,
					Box::new(lhs),
					Box::new(rhs));
			}
			else
				{ break; }
		}
		
		Ok(lhs)
	}
	

	fn parse_right_associative_binary_ops<F>(&mut self, ops: &[(syntax::TokenKind, expr::BinaryOp)], parse_inner: F) -> Result<expr::Expr, ()>
	where F: Fn(&mut ExpressionParser<'a, 'src>) -> Result<expr::Expr, ()>
	{
		let mut lhs = parse_inner(self)?;
		
		let mut op_match = None;
		
		for op in ops
		{
			if let Some(tk) = self.walker.maybe_expect(op.0)
			{
				op_match = Some((tk.span, op.1));
				break;
			}
		}
		
		if let Some(op_match) = op_match
		{				
			let rhs = self.parse_expr()?;
			let span = lhs.span().join(rhs.span());
			
			lhs = expr::Expr::BinaryOp(span, op_match.0, op_match.1, Box::new(lhs), Box::new(rhs));
		}
		
		Ok(lhs)
	}
	
	
	fn parse_ternary_conditional(&mut self) -> Result<expr::Expr, ()>
	{
		let cond = self.parse_assignment()?;
		
		if self.walker.maybe_expect(syntax::TokenKind::Question).is_some()
		{
			let true_branch = self.parse_expr()?;
			
			let false_branch =
			{
				if self.walker.maybe_expect(syntax::TokenKind::Colon).is_some()
					{ self.parse_expr()? }
				else
					{ expr::Expr::Block(true_branch.span(), Vec::new()) }
			};
			
			Ok(expr::Expr::TernaryOp(cond.span().join(false_branch.span()), Box::new(cond), Box::new(true_branch), Box::new(false_branch)))
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
				(syntax::TokenKind::DoubleVerticalBar, expr::BinaryOp::LazyOr)
			],
			|s| s.parse_lazy_and())
	}
	
	
	fn parse_lazy_and(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::DoubleAmpersand, expr::BinaryOp::LazyAnd)
			],
			|s| s.parse_relational())
	}
	
	
	fn parse_relational(&mut self) -> Result<expr::Expr, ()>
	{
		self.parse_binary_ops(
			&[
				(syntax::TokenKind::DoubleEqual,       expr::BinaryOp::Eq),
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
				(syntax::TokenKind::DoubleLessThan,       expr::BinaryOp::Shl),
				(syntax::TokenKind::DoubleGreaterThan, expr::BinaryOp::Shr)
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
			|s| s.parse_slice())
	}
	
	
	fn parse_slice(&mut self) -> Result<expr::Expr, ()>
	{
		let inner = self.parse_slice_short()?;
		
		if self.walker.next_linebreak().is_some()
			{ return Ok(inner); }

		let tk_open = match self.walker.maybe_expect(syntax::TokenKind::BracketOpen)
		{
			Some(tk) => tk,
			None => return Ok(inner)
		};
			
		let leftmost = self.parse_expr()?;

        self.walker.expect(self.report, syntax::TokenKind::Colon)?;
		
		let rightmost = self.parse_expr()?;

        let tk_close_span = self.walker
			.expect(self.report, syntax::TokenKind::BracketClose)?
            .span;
		
		let slice_span = tk_open.span.join(tk_close_span);
		let span = inner.span().join(tk_close_span);
		
		Ok(expr::Expr::Slice(
			span,
			slice_span,
			Box::new(leftmost),
			Box::new(rightmost),
			Box::new(inner)))
	}
	
	
	fn parse_slice_short(&mut self) -> Result<expr::Expr, ()>
	{
		let inner = self.parse_unary()?;
		
		if self.walker.next_linebreak().is_some()
			{ return Ok(inner); }
		
		let tk_grave_span = match self.walker.maybe_expect(syntax::TokenKind::Grave)
		{
			Some(tk) => tk.span,
			None => return Ok(inner)
		};

		let size = self.parse_leaf()?;

		Ok(expr::Expr::SliceShort(
			tk_grave_span.join(size.span()),
			size.span(),
			Box::new(size),
			Box::new(inner)))
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
		
		if self.walker.next_linebreak().is_some()
			{ return Ok(leaf); }
			
		if self.walker.maybe_expect(syntax::TokenKind::ParenOpen).is_none()
			{ return Ok(leaf); }
			
		let mut args = Vec::new();
		while !self.walker.next_useful_is(0, syntax::TokenKind::ParenClose)
		{
			args.push(self.parse_expr()?);
			
			if self.walker.next_useful_is(0, syntax::TokenKind::ParenClose)
				{ break; }
				
			self.walker.expect(self.report, syntax::TokenKind::Comma)?;
		}
		
		let tk_close = self.walker.expect(self.report, syntax::TokenKind::ParenClose)?;
		
		Ok(expr::Expr::Call(leaf.span().join(tk_close.span), Box::new(leaf), args))
	}
	
	
	fn parse_leaf(&mut self) -> Result<expr::Expr, ()>
	{
		if self.walker.next_useful_is(0, syntax::TokenKind::BraceOpen)
			{ self.parse_block() }
	
		else if self.walker.next_useful_is(0, syntax::TokenKind::ParenOpen)
			{ self.parse_parenthesized() }
	
		else if self.walker.next_useful_is(0, syntax::TokenKind::Identifier)
			{ self.parse_variable() }
			
		else if self.walker.next_useful_is(0, syntax::TokenKind::Dot)
			{ self.parse_variable() }
			
		else if self.walker.next_useful_is(0, syntax::TokenKind::Number)
			{ self.parse_number() }
			
		else if self.walker.next_useful_is(0, syntax::TokenKind::String)
			{ self.parse_string() }
	
		else if self.walker.next_useful_is(0, syntax::TokenKind::KeywordAsm)
			{ self.parse_asm() }
	
		else if self.walker.next_useful_is(0, syntax::TokenKind::KeywordTrue)
			{ self.parse_boolean_true() }
	
		else if self.walker.next_useful_is(0, syntax::TokenKind::KeywordFalse)
			{ self.parse_boolean_false() }
			
		else
		{
            self.report.error_span(
                "expected expression",
                self.walker.get_cursor_span());
            
			Err(())
		}
	}
	
	
	fn parse_block(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_open_span = self.walker
			.expect(self.report, syntax::TokenKind::BraceOpen)?
            .span;
		
		let mut exprs = Vec::new();
		while !self.walker.next_useful_is(0, syntax::TokenKind::BraceClose)
		{
			exprs.push(self.parse_expr()?);
			
			if self.walker.maybe_expect_linebreak().is_some()
				{ continue; }
				
			if self.walker.next_useful_is(0, syntax::TokenKind::BraceClose)
				{ break; }
				
			self.walker.expect(self.report, syntax::TokenKind::Comma)?;
		}
		
		let tk_close = self.walker.expect(self.report, syntax::TokenKind::BraceClose)?;
		
		Ok(expr::Expr::Block(tk_open_span.join(tk_close.span), exprs))
	}
	
	
	fn parse_parenthesized(&mut self) -> Result<expr::Expr, ()>
	{
		self.walker.expect(self.report, syntax::TokenKind::ParenOpen)?;
		let expr = self.parse_expr()?;
		self.walker.expect(self.report, syntax::TokenKind::ParenClose)?;
		Ok(expr)
	}
	
	
	fn parse_variable(&mut self) -> Result<expr::Expr, ()>
	{
		let mut span = diagn::Span::new_dummy();
		let mut hierarchy_level = 0;

		loop
		{
			if self.walker.next_linebreak().is_some()
			{
				break;
			}

			if let Some(tk_dot) = self.walker.maybe_expect(syntax::TokenKind::Dot)
			{
				hierarchy_level += 1;
				span = span.join(tk_dot.span);
				continue;
			}

			break;
		}

		let mut hierarchy = Vec::new();

		loop
		{
			let tk_name = self.walker.expect(self.report, syntax::TokenKind::Identifier)?;
			let name = self.walker.get_span_excerpt(tk_name.span).to_string();
			hierarchy.push(name);
			span = span.join(tk_name.span);

			if self.walker.next_linebreak().is_some()
			{
				break;
			}
			
			if self.walker.maybe_expect(syntax::TokenKind::Dot).is_none()
			{
				break;
			}
		}
		
		Ok(expr::Expr::Variable(span, hierarchy_level, hierarchy))
	}
	
	
	fn parse_number(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_number = self.walker.expect(self.report, syntax::TokenKind::Number)?;
		let number = self.walker.get_span_excerpt(tk_number.span);
		
		let bigint = syntax::excerpt_as_bigint(
			Some(self.report),
			tk_number.span,
			number)?;
		
		let expr = expr::Expr::Literal(
			tk_number.span,
			expr::Value::Integer(bigint));

		Ok(expr)
	}
	
	
	fn parse_boolean_true(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_true = self.walker.expect(
			self.report,
			syntax::TokenKind::KeywordTrue)?;

		let expr = expr::Expr::Literal(
			tk_true.span,
			expr::Value::Bool(true));

		Ok(expr)
	}
	
	
	fn parse_boolean_false(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_true = self.walker.expect(
			self.report,
			syntax::TokenKind::KeywordFalse)?;

		let expr = expr::Expr::Literal(
			tk_true.span,
			expr::Value::Bool(false));

		Ok(expr)
	}
	
	
	fn parse_string(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_str = self.walker.expect(self.report, syntax::TokenKind::String)?;

		let string = syntax::excerpt_as_string_contents(
			self.report,
			tk_str.span,
			self.walker.get_span_excerpt(tk_str.span))?;
		
		let expr = expr::Expr::Literal(
			tk_str.span,
			expr::Value::String(expr::ExprString
			{
				utf8_contents: string,
				encoding: "utf8".to_string(),
			}));

		Ok(expr)
	}
	
	
	fn parse_asm(&mut self) -> Result<expr::Expr, ()>
	{
		let tk_asm = self.walker.expect(
			self.report,
			syntax::TokenKind::KeywordAsm)?;
    
		let _tk_brace_open = self.walker.expect(
			self.report,
			syntax::TokenKind::BraceOpen)?;

		let mut inner_walker = self.walker
			.advance_until_closing_brace();

		let ast = asm::parser::parse_nested_toplevel(
			self.report,
			&mut inner_walker)?;

		let tk_brace_close = self.walker.expect(
			self.report,
			syntax::TokenKind::BraceClose)?;

		let expr = expr::Expr::Asm(
			tk_asm.span.join(tk_brace_close.span),
			ast);

		Ok(expr)
	}
}