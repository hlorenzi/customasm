use diagn::{Span, Report};
use syntax::{Token, TokenKind, tokenize, Parser};
use syntax::excerpt_as_string_contents;
use expr::{Expression, ExpressionValue};
use asm::{AssemblerState, ParsedInstruction, ParsedExpression};
use util::filename_navigate;
use num::BigInt;
use num::ToPrimitive;


pub struct AssemblerParser<'a, 'b>
where 'b: 'a
{
	pub state: &'a mut AssemblerState<'b>,
	pub cur_filename: String,
	pub parser: Parser<'a>
}
	
	
impl<'a, 'b> AssemblerParser<'a, 'b>
{
	pub fn parse_file<S>(report: &mut Report, state: &mut AssemblerState, filename: S, filename_span: Option<&Span>) -> Result<(), ()>
	where S: Into<String>
	{
		let filename_owned = filename.into();
		let chars = state.fileserver.get_chars(report, &filename_owned, filename_span)?;
		let tokens = tokenize(report, filename_owned.as_ref(), &chars)?;
		
		let mut parser = AssemblerParser
		{
			state: state,
			cur_filename: filename_owned,
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
		
		if name.chars().next() == Some('d')
		{
			if let Ok(elem_width) = usize::from_str_radix(&name[1..], 10)
				{ return self.parse_directive_data(elem_width, &tk_name); }
		}
		
		match name.as_ref()
		{
			"addr"      => self.parse_directive_addr(&tk_name),
			"outp"      => self.parse_directive_outp(&tk_name),
			"res"       => self.parse_directive_res(&tk_name),
			"include"   => self.parse_directive_include(),
			"incbin"    => self.parse_directive_incbin(&tk_name),
			"incbinstr" => self.parse_directive_incstr(1, &tk_name),
			"inchexstr" => self.parse_directive_incstr(4, &tk_name),
			_ => Err(self.parser.report.error_span("unknown directive", &tk_name.span))
		}
	}
	
	
	fn parse_directive_addr(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let address_byte = self.parse_usize()?;
		
		match address_byte.checked_mul(self.state.instrset.align)
		{
			Some(address_bit) => self.state.cur_address_bit = address_bit,
			None => return Err(self.parser.report.error_span("address is out of valid range", &tk_name.span))
		}
		
		Ok(())
	}
	
	
	fn parse_directive_outp(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let output_byte = self.parse_usize()?;
		
		match output_byte.checked_mul(self.state.instrset.align)
		{
			Some(output_bit) => self.state.cur_output_bit = output_bit,
			None => return Err(self.parser.report.error_span("output pointer is out of valid range", &tk_name.span))
		}
		
		Ok(())
	}
	
	
	fn parse_directive_res(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let bits = self.parse_usize()? * self.state.instrset.align;
		
		self.state.output_zero_bits(self.parser.report, bits, &tk_name.span)
	}
	
	
	fn parse_directive_include(&mut self) -> Result<(), ()>
	{
		let tk_filename = self.parser.expect(TokenKind::String)?;
		let filename = excerpt_as_string_contents(self.parser.report, tk_filename.excerpt.as_ref().unwrap().as_ref(), &tk_filename.span)?;
	
		let new_filename = filename_navigate(self.parser.report, &self.cur_filename, &filename, &tk_filename.span)?;
		
		AssemblerParser::parse_file(self.parser.report, self.state, new_filename, Some(&tk_filename.span))
	}
	
	
	fn parse_directive_incbin(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let tk_filename = self.parser.expect(TokenKind::String)?;
		let filename = excerpt_as_string_contents(self.parser.report, tk_filename.excerpt.as_ref().unwrap().as_ref(), &tk_filename.span)?;
		
		let new_filename = filename_navigate(self.parser.report, &self.cur_filename, &filename, &tk_filename.span)?;
		
		let bytes = self.state.fileserver.get_bytes(self.parser.report, &new_filename, Some(&tk_filename.span))?;
		
		for mut byte in bytes
		{
			for _ in 0..8
			{
				let bit = byte & 0x80 != 0;
				self.state.output_bit(self.parser.report, bit, &tk_filename.span)?;
				byte <<= 1;
			}
		}
		
		let excess_bits = self.state.cur_address_bit % self.state.instrset.align;
		if excess_bits != 0
			{ return Err(self.parser.report.error_span(format!("data leaves address misaligned (excess bits = {})", excess_bits), &tk_name.span)); }
		
		Ok(())
	}
	
	
	fn parse_directive_incstr(&mut self, bits_per_char: usize, tk_name: &Token) -> Result<(), ()>
	{
		let tk_filename = self.parser.expect(TokenKind::String)?;
		let filename = excerpt_as_string_contents(self.parser.report, tk_filename.excerpt.as_ref().unwrap().as_ref(), &tk_filename.span)?;
		
		let new_filename = filename_navigate(self.parser.report, &self.cur_filename, &filename, &tk_filename.span)?;
		
		let chars = self.state.fileserver.get_chars(self.parser.report, &new_filename, Some(&tk_filename.span))?;
		
		for c in chars
		{
			let mut digit = match c.to_digit(1 << bits_per_char)
			{
				Some(digit) => digit,
				None => return Err(self.parser.report.error_span("invalid character in file contents", &tk_filename.span))
			};
			
			for _ in 0..bits_per_char
			{
				let bit = digit & (1 << (bits_per_char - 1)) != 0;
				self.state.output_bit(self.parser.report, bit, &tk_filename.span)?;
				digit <<= 1;
			}
		}
		
		let excess_bits = self.state.cur_address_bit % self.state.instrset.align;
		if excess_bits != 0
			{ return Err(self.parser.report.error_span(format!("data leaves address misaligned (excess bits = {})", excess_bits), &tk_name.span)); }
		
		Ok(())
	}
	
	
	fn parse_directive_data(&mut self, elem_width: usize, tk_name: &Token) -> Result<(), ()>
	{
		if elem_width == 0
			{ return Err(self.parser.report.error_span("invalid element width", &tk_name.span)); }
			
		loop
		{
			let ctx = self.state.get_cur_context();
			let expr = Expression::parse(&mut self.parser)?;
			let span = expr.span();
			
			let parsed_expr = ParsedExpression
			{
				ctx: ctx,
				width: elem_width,
				expr: expr
			};
			
			self.state.parsed_exprs.push(parsed_expr);
			
			self.state.output_zero_bits(self.parser.report, elem_width, &span)?;
			
			if self.parser.maybe_expect(TokenKind::Comma).is_none()
				{ break; }
		}
		
		let excess_bits = self.state.cur_address_bit % self.state.instrset.align;
		if excess_bits != 0
			{ return Err(self.parser.report.error_span(format!("data leaves address misaligned (excess bits = {})", excess_bits), &tk_name.span)); }
		
		Ok(())
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
			ExpressionValue::Integer(BigInt::from(self.state.cur_address_bit / self.state.instrset.align))
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
		// Also output zero bits to advance address and output pointer.
		let rule = &self.state.instrset.rules[instr_match.rule_indices[best_match]];
		
		let instr_width = rule.production.width().unwrap();
		self.state.output_zero_bits(self.parser.report, instr_width, &instr_span)?;
		
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
		let ctx = self.state.get_cur_context();
		
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