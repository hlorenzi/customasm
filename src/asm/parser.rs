use diagn::{Span, RcReport};
use syntax::{Token, TokenKind, tokenize, Parser};
use syntax::excerpt_as_string_contents;
use expr::{Expression, ExpressionValue};
use asm::{AssemblerState, ParsedInstruction, ParsedExpression};
use asm::cpudef::CpuDef;
use asm::BankDef;
use asm::BinaryBlock;
use util::filename_navigate;
use util::FileServer;
use num::BigInt;
use num::ToPrimitive;


pub struct AssemblerParser<'a>
{
	pub fileserver: &'a FileServer,
	pub state: &'a mut AssemblerState,
	pub cur_filename: String,
	pub parser: Parser
}
	
	
impl<'a> AssemblerParser<'a>
{
	pub fn parse_file<S>(report: RcReport, state: &mut AssemblerState, fileserver: &FileServer, filename: S, filename_span: Option<&Span>) -> Result<(), ()>
	where S: Into<String>
	{
		let filename_owned = filename.into();
		let chars = fileserver.get_chars(report.clone(), &filename_owned, filename_span)?;
		let tokens = tokenize(report.clone(), filename_owned.as_ref(), &chars)?;
		
		let mut parser = AssemblerParser
		{
			fileserver: fileserver,
			state: state,
			cur_filename: filename_owned,
			parser: Parser::new(report.clone(), tokens)
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
			"cpudef"    => self.parse_directive_cpudef(&tk_name),
			"bankdef"   => self.parse_directive_bankdef(&tk_name),
			"bank"      => self.parse_directive_bank(&tk_name),
			"addr"      => self.parse_directive_addr(&tk_name),
			"res"       => self.parse_directive_res(&tk_name),
			"str"       => self.parse_directive_str(&tk_name),
			"include"   => self.parse_directive_include(),
			"incbin"    => self.parse_directive_incbin(&tk_name),
			"incbinstr" => self.parse_directive_incstr(1, &tk_name),
			"inchexstr" => self.parse_directive_incstr(4, &tk_name),
			_ => Err(self.parser.report.error_span("unknown directive", &tk_name.span))
		}
	}
	
	
	fn parse_directive_cpudef(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		self.parser.maybe_expect(TokenKind::String);
		
		self.parser.expect(TokenKind::BraceOpen)?;
		
		if self.state.cpudef.is_some()
			{ return Err(self.parser.report.error_span("cpu already set", &tk_name.span)); }
		
		self.state.cpudef = Some(CpuDef::parse(&mut self.parser)?);
		
		self.parser.expect(TokenKind::BraceClose)?;
		
		Ok(())
	}
	
	
	fn parse_directive_bankdef(&mut self, _tk_name: &Token) -> Result<(), ()>
	{
		let tk_bankname = self.parser.expect(TokenKind::String)?;
		let bankname = excerpt_as_string_contents(self.parser.report.clone(), tk_bankname.excerpt.as_ref().unwrap().as_ref(), &tk_bankname.span)?;
		
		if bankname == ""
			{ return Err(self.parser.report.error_span("invalid bank name", &tk_bankname.span)); }
		
		if self.state.find_bankdef(&bankname).is_some()
			{ return Err(self.parser.report.error_span("duplicate bank name", &tk_bankname.span)); }
			
		if self.state.blocks[0].len() > 0
			{ return Err(self.parser.report.error_span("cannot define bank after using the default bank", &tk_bankname.span)); }
		
		self.parser.expect(TokenKind::BraceOpen)?;
		let bankdef = BankDef::parse(bankname.clone(), &mut self.parser, &tk_bankname.span)?;
		self.parser.expect(TokenKind::BraceClose)?;
		
		self.state.bankdefs.push(bankdef);
		self.state.blocks.push(BinaryBlock::new(bankname));
		self.state.cur_bank = self.state.bankdefs.len() - 1;
		self.state.cur_block = self.state.bankdefs.len() - 1;
		Ok(())
	}
	
	
	fn parse_directive_bank(&mut self, _tk_name: &Token) -> Result<(), ()>
	{
		let tk_bankname = self.parser.expect(TokenKind::String)?;
		let bankname = excerpt_as_string_contents(self.parser.report.clone(), tk_bankname.excerpt.as_ref().unwrap().as_ref(), &tk_bankname.span)?;
		
		let bank_index = match self.state.find_bankdef(&bankname)
		{
			Some(i) => i,
			None => return Err(self.parser.report.error_span("unknown bank", &tk_bankname.span))
		};
		
		self.state.cur_bank = bank_index;
		self.state.cur_block = bank_index;
		Ok(())
	}
	
	
	fn parse_directive_addr(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let new_addr = self.parse_usize()?;
		
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
		let cur_addr = self.state.get_cur_address(self.parser.report.clone(), &tk_name.span)?;
			
		self.state.check_valid_address(self.parser.report.clone(), self.state.cur_block, new_addr, &tk_name.span)?;
		
		if new_addr < cur_addr
			{ return Err(self.parser.report.error_span("cannot seek to previous address", &tk_name.span)); }
			
		let bits_to_skip = (new_addr - cur_addr) * self.state.cpudef.as_ref().unwrap().align;
		self.state.output_zero_bits(self.parser.report.clone(), bits_to_skip, &tk_name.span)
	}
	
	
	fn parse_directive_res(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
		let bits = self.parse_usize()? * self.state.cpudef.as_ref().unwrap().align;
		
		self.state.output_zero_bits(self.parser.report.clone(), bits, &tk_name.span)
	}
	
	
	fn parse_directive_str(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
		let tk_string = self.parser.expect(TokenKind::String)?;
		let string = excerpt_as_string_contents(self.parser.report.clone(), tk_string.excerpt.as_ref().unwrap().as_ref(), &tk_string.span)?;
		
		for mut byte in string.bytes()
		{
			for _ in 0..8
			{
				let bit = byte & 0x80 != 0;
				self.state.output_bit(self.parser.report.clone(), bit, &tk_string.span)?;
				byte <<= 1;
			}
		}
		
		Ok(())
	}
	
	
	fn parse_directive_include(&mut self) -> Result<(), ()>
	{
		let tk_filename = self.parser.expect(TokenKind::String)?;
		let filename = excerpt_as_string_contents(self.parser.report.clone(), tk_filename.excerpt.as_ref().unwrap().as_ref(), &tk_filename.span)?;
	
		let new_filename = filename_navigate(self.parser.report.clone(), &self.cur_filename, &filename, &tk_filename.span)?;
		
		AssemblerParser::parse_file(self.parser.report.clone(), self.state, self.fileserver, new_filename, Some(&tk_filename.span))
	}
	
	
	fn parse_directive_incbin(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
		let tk_filename = self.parser.expect(TokenKind::String)?;
		let filename = excerpt_as_string_contents(self.parser.report.clone(), tk_filename.excerpt.as_ref().unwrap().as_ref(), &tk_filename.span)?;
		
		let new_filename = filename_navigate(self.parser.report.clone(), &self.cur_filename, &filename, &tk_filename.span)?;
		
		let bytes = self.fileserver.get_bytes(self.parser.report.clone(), &new_filename, Some(&tk_filename.span))?;
		
		for mut byte in bytes
		{
			for _ in 0..8
			{
				let bit = byte & 0x80 != 0;
				self.state.output_bit(self.parser.report.clone(), bit, &tk_filename.span)?;
				byte <<= 1;
			}
		}
		
		Ok(())
	}
	
	
	fn parse_directive_incstr(&mut self, bits_per_char: usize, tk_name: &Token) -> Result<(), ()>
	{
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
		let tk_filename = self.parser.expect(TokenKind::String)?;
		let filename = excerpt_as_string_contents(self.parser.report.clone(), tk_filename.excerpt.as_ref().unwrap().as_ref(), &tk_filename.span)?;
		
		let new_filename = filename_navigate(self.parser.report.clone(), &self.cur_filename, &filename, &tk_filename.span)?;
		
		let chars = self.fileserver.get_chars(self.parser.report.clone(), &new_filename, Some(&tk_filename.span))?;
		
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
				self.state.output_bit(self.parser.report.clone(), bit, &tk_filename.span)?;
				digit <<= 1;
			}
		}
		
		Ok(())
	}
	
	
	fn parse_directive_data(&mut self, elem_width: usize, tk_name: &Token) -> Result<(), ()>
	{
		if elem_width == 0
			{ return Err(self.parser.report.error_span("invalid element width", &tk_name.span)); }
		
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
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
			
			self.state.output_zero_bits(self.parser.report.clone(), elem_width, &span)?;
			
			if self.parser.maybe_expect(TokenKind::Comma).is_none()
				{ break; }
		}
		
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
			let value = self.state.expr_eval(self.parser.report.clone(), &ctx, &expr)?;
			self.parser.expect_linebreak()?;
			value
		}
		else
		{
			self.parser.expect(TokenKind::Colon)?;
			
			let addr = self.state.get_cur_address(self.parser.report.clone(), &tk_name.span)?;
			ExpressionValue::Integer(BigInt::from(addr))
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
		self.state.check_cpudef_active(self.parser.report.clone(), &self.parser.next().span)?;
		
		let instr_span_start = self.parser.next().span;
		
		// Find matching rule patterns.
		self.parser.clear_linebreak();
		let instr_match = match self.state.cpudef.as_ref().unwrap().pattern_matcher.parse_match(&mut self.parser)
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
			// Use a dummy report to not propagate errors now.
			args.push(self.state.expr_eval(RcReport::new(), &ctx, expr).ok());
		}
		
		// If there is more than one match, find best suited match.
		let best_match =
		{
			let mut best_match = 0;
			while best_match < instr_match.rule_indices.len() - 1
			{
				// Check rule constraints. If it relies on an argument that could not
				// be resolved now, of if it fails, just skip this rule without an error.
				let rule = &self.state.cpudef.as_ref().unwrap().rules[instr_match.rule_indices[best_match]];
				let get_arg = |i: usize| args[i].clone();
				if self.state.rule_check_all_constraints_satisfied(RcReport::new(), rule, &get_arg, &ctx, &instr_span).ok().is_some()
					{ break; }
					
				best_match += 1;
			}
			
			best_match
		};
		
		// Having found the best matching rule, save it to be output on the second pass.
		// Remaining argument resolution and constraint checking will be done then.
		// Also output zero bits to advance address and output pointer.
		let instr_width =
		{
			let rule = &self.state.cpudef.as_ref().unwrap().rules[instr_match.rule_indices[best_match]];
			rule.production.width().unwrap()
		};
		
		self.state.output_zero_bits(self.parser.report.clone(), instr_width, &instr_span)?;
		
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
		
		let value = match self.state.expr_eval(self.parser.report.clone(), &ctx, &expr)
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