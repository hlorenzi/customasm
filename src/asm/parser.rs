use diagn::Report;
use syntax::{Token, TokenKind, tokenize, Parser};
use expr::{Expression, ExpressionValue};
use asm::{AssemblerState, ParsedInstruction, ExpressionContext};
use num::BigInt;
use num::ToPrimitive;


pub struct AssemblerParser<'a, 'b>
where 'b: 'a
{
	pub state: &'a mut AssemblerState<'b>,
	pub parser: Parser<'a>
}
	
	
impl<'a, 'b> AssemblerParser<'a, 'b>
{
	pub fn parse_file<S>(report: &mut Report, state: &mut AssemblerState, filename: S) -> Result<(), ()>
	where S: Into<String>
	{
		let filename_owned = filename.into();
		let chars = state.fileserver.get_chars(report, &filename_owned)?;
		let tokens = tokenize(report, filename_owned, &chars)?;
		
		let mut parser = AssemblerParser
		{
			state: state,
			parser: Parser::new(report, &tokens)
		};
		
		parser.parse()
	}
	
	
	fn parse(&mut self) -> Result<(), ()>
	{
		while !self.parser.is_over()
		{
			self.parse_line()?;
		}
		
		Ok(())
	}
	
	
	fn parse_line(&mut self) -> Result<(), ()>
	{
		if self.parser.next_is(0, TokenKind::Hash)
		{
			self.parse_directive()?;
			self.parser.expect_linebreak()
		}
			
		else if self.parser.next_is(0, TokenKind::Identifier) && self.parser.next_is(1, TokenKind::Colon)
			{ self.parse_label() }
			
		else if self.parser.next_is(0, TokenKind::Identifier) && self.parser.next_is(1, TokenKind::Equal)
			{ self.parse_label() }
	
		else if self.parser.next_is(0, TokenKind::Dot) && self.parser.next_is(1, TokenKind::Identifier) && self.parser.next_is(2, TokenKind::Colon)
			{ self.parse_label() }
			
		else if self.parser.next_is(0, TokenKind::Dot) && self.parser.next_is(1, TokenKind::Identifier) && self.parser.next_is(2, TokenKind::Equal)
			{ self.parse_label() }
		
		else
		{
			self.parse_instruction()?;
			self.parser.expect_linebreak()
		}
	}
	
	
	fn parse_directive(&mut self) -> Result<(), ()>
	{
		self.parser.expect(TokenKind::Hash)?;
		
		let tk_name = self.parser.expect(TokenKind::Identifier)?;
		let name = tk_name.excerpt.clone().unwrap();
		
		match name.as_ref()
		{
			"addr" => self.parse_directive_addr(),
			"outp" => self.parse_directive_outp(),
			"res"  => self.parse_directive_res(&tk_name),
			_ => Err(self.parser.report.error_span("unknown directive", &tk_name.span))
		}
	}
	
	
	fn parse_directive_addr(&mut self) -> Result<(), ()>
	{
		self.state.cur_address = self.parse_usize()?;
		Ok(())
	}
	
	
	fn parse_directive_outp(&mut self) -> Result<(), ()>
	{
		self.state.cur_writehead = self.parse_usize()?;
		Ok(())
	}
	
	
	fn parse_directive_res(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let bytes = self.parse_usize()?;
		self.state.output_zeroes(self.parser.report, bytes, &tk_name.span)
	}
	
	
	fn parse_label(&mut self) -> Result<(), ()>
	{
		let is_local = self.parser.maybe_expect(TokenKind::Dot).is_some();
		let mut name = if is_local { "." } else { "" }.to_string();
	
		let tk_name = self.parser.expect(TokenKind::Identifier)?;
		name.push_str(&tk_name.excerpt.unwrap());
		
		let ctx = self.state.get_cur_context();
		
		let value = if self.parser.maybe_expect(TokenKind::Equal).is_some()
		{		
			let expr = Expression::parse(&mut self.parser)?;
			let value = self.state.expr_eval(self.parser.report, &ctx, &expr)?;
			self.parser.expect_linebreak()?;
			value
		}
		else
		{
			self.parser.expect(TokenKind::Colon)?;
			ExpressionValue::Integer(BigInt::from(self.state.cur_address))
		};

		if is_local
		{
			if self.state.labels.local_exists(ctx.label_ctx, &name)
				{ return Err(self.parser.report.error_span("duplicate local label", &tk_name.span)); }
				
			self.state.labels.add_local(ctx.label_ctx, name, value);
		}
		else
		{
			if self.state.labels.global_exists(&name)
				{ return Err(self.parser.report.error_span("duplicate global label", &tk_name.span)); }
				
			self.state.labels.add_global(name, value);
		}
		
		Ok(())
	}
	
	
	fn parse_instruction(&mut self) -> Result<(), ()>
	{
		let instr_span_start = self.parser.next().span;
		
		// Find matching rule patterns.
		self.parser.clear_linebreak();
		let instr_match = match self.state.pattern_matcher.parse_match(&mut self.parser)
		{
			Some(m) => m,
			None =>
			{
				// Just skip instruction and continue parsing ahead.
				self.parser.skip_until_linebreak();
				let instr_span = instr_span_start.join(&self.parser.prev().span);
				
				self.parser.report.error_span("no match for instruction found", &instr_span);
				return Ok(());
			}
		};
		
		let instr_span = instr_span_start.join(&self.parser.prev().span);
		
		let ctx = self.state.get_cur_context();
		
		// Resolve as many arguments as possible right now.
		let mut args: Vec<Option<ExpressionValue>> = Vec::new();
		
		for expr in &instr_match.exprs
		{
			// Do not report or propagate errors now.
			let mut dummy_report = Report::new();
			args.push(self.state.expr_eval(&mut dummy_report, &ctx, expr).ok());
		}
		
		// If there is more than one match, find best suited match.
		let best_match =
		{
			let mut best_match = 0;
			while best_match < instr_match.rule_indices.len() - 1
			{
				// Check rule constraints. If it relies on an argument that could not
				// be resolved now, of if it fails, just skip this rule without an error.
				let rule = &self.state.instrset.rules[instr_match.rule_indices[best_match]];
				let get_arg = |i: usize| args[i].clone();
				let mut dummy_report = Report::new();
				if self.state.rule_check_all_constraints_satisfied(&mut dummy_report, rule, &get_arg, &ctx, &instr_span).ok().is_some()
					{ break; }
					
				best_match += 1;
			}
			
			best_match
		};
		
		// Having found the best matching rule, save it to be output on the second pass.
		// Remaining argument resolution and constraint checking will be done then.
		// Also advance address and write pointer.
		let rule = &self.state.instrset.rules[instr_match.rule_indices[best_match]];
		
		let instr_width = rule.production_width() / self.state.instrset.align;
		self.state.output_advance(self.parser.report, instr_width, &instr_span)?;
		
		let parsed_instr = ParsedInstruction
		{
			rule_index: instr_match.rule_indices[best_match],
			span: instr_span,
			ctx: ctx,
			exprs: instr_match.exprs,
			args: args
		};
		
		self.state.parsed_instrs.push(parsed_instr);
		
		Ok(())
	}
	
	
	fn parse_usize(&mut self) -> Result<usize, ()>
	{
		let ctx = ExpressionContext
		{
			label_ctx: self.state.labels.get_cur_context(),
			address: self.state.cur_address,
			writehead: self.state.cur_writehead
		};
		
		let expr = Expression::parse(&mut self.parser)?;
		
		let value = match self.state.expr_eval(self.parser.report, &ctx, &expr)
		{
			Ok(ExpressionValue::Integer(value)) => value,
			Ok(_) => return Err(self.parser.report.error_span("expected integer value", &expr.span())),
			Err(()) => return Err(())
		};
		
		match value.to_usize()
		{
			Some(x) => Ok(x),
			None => Err(self.parser.report.error_span("value is too large", &expr.span()))
		}
	}
}