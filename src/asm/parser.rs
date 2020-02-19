use crate::diagn::{Span, RcReport};
use crate::syntax::{Token, TokenKind, tokenize, Parser};
use crate::syntax::excerpt_as_string_contents;
use crate::expr::{Expression, ExpressionValue, ExpressionEvalContext};
use crate::asm::{AssemblerState, ParsedInstruction, ParsedExpression};
use crate::asm::cpudef::CpuDef;
use crate::asm::BankDef;
use crate::asm::Bank;
use crate::util::filename_navigate;
use crate::util::FileServer;
use num_bigint::BigInt;
use num_traits::ToPrimitive;


pub struct AssemblerParser<'a>
{
	pub fileserver: &'a dyn FileServer,
	pub state: &'a mut AssemblerState,
	pub cur_filename: String,
	pub parser: Parser
}
	
	
impl<'a> AssemblerParser<'a>
{
	pub fn parse_file<S>(report: RcReport, state: &mut AssemblerState, fileserver: &dyn FileServer, filename: S, filename_span: Option<&Span>) -> Result<(), ()>
	where S: Into<String>
	{
		let filename_owned = filename.into();
		let chars = fileserver.get_chars(report.clone(), &filename_owned, filename_span)?;
		let tokens = tokenize(report.clone(), AsRef::<str>::as_ref(&filename_owned), &chars)?;
		
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
		let tk_hash = self.parser.expect(TokenKind::Hash)?;
		
		let tk_name = self.parser.expect(TokenKind::Identifier)?;
		let name = tk_name.excerpt.clone().unwrap();
		
		if name.chars().next() == Some('d')
		{
			if let Ok(elem_width) = usize::from_str_radix(&name[1..], 10)
				{ return self.parse_directive_data(elem_width, &tk_hash, &tk_name); }
		}
		
		match name.as_ref()
		{
			"cpudef"    => self.parse_directive_cpudef(&tk_name),
			"bankdef"   => self.parse_directive_bankdef(&tk_name),
			"bank"      => self.parse_directive_bank(&tk_name),
			"addr"      => self.parse_directive_addr(&tk_name),
			"align"     => self.parse_directive_align(&tk_name),
			"res"       => self.parse_directive_res(&tk_name),
			"str"       => self.parse_directive_str(),
			"include"   => self.parse_directive_include(),
			"incbin"    => self.parse_directive_incbin(),
			"incbinstr" => self.parse_directive_incstr(1),
			"inchexstr" => self.parse_directive_incstr(4),
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
			
		if self.state.blocks[0].bits.len() > 0
			{ return Err(self.parser.report.error_span("cannot define bank after using the default bank", &tk_bankname.span)); }
		
		self.parser.expect(TokenKind::BraceOpen)?;
		let bankdef = BankDef::parse(bankname.clone(), &mut self.parser, &tk_bankname.span)?;
		self.parser.expect(TokenKind::BraceClose)?;
		
		self.state.bankdefs.push(bankdef);
		self.state.blocks.push(Bank::new(bankname));
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
		
		let bits = self.state.cpudef.as_ref().unwrap().bits;
		
		if new_addr < cur_addr
		{
			let bankdef = &self.state.bankdefs[self.state.cur_bank];
			
			if bankdef.outp.is_some()
				{ return Err(self.parser.report.error_span("cannot seek to previous address", &tk_name.span)); }
			
			self.state.blocks[self.state.cur_block].bits.truncate(new_addr * bits);
			Ok(())
		}
		else
		{
			let bits_to_skip = (new_addr - cur_addr) * bits;
			self.state.output_zero_bits(self.parser.report.clone(), bits_to_skip, true, &tk_name.span, None)
		}
	}
	
	
	fn parse_directive_align(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
		let addr_multiple_of = self.parse_usize()?;
		
		self.state.output_bits_until_aligned(self.parser.report.clone(), addr_multiple_of, &tk_name.span, None)
	}
	
	
	fn parse_directive_res(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		self.state.check_cpudef_active(self.parser.report.clone(), &tk_name.span)?;
		
		let bits = self.parse_usize()? * self.state.cpudef.as_ref().unwrap().bits;
		
		self.state.output_zero_bits(self.parser.report.clone(), bits, true, &tk_name.span, Some(&tk_name.span))
	}
	
	
	fn parse_directive_str(&mut self) -> Result<(), ()>
	{
		let tk_string = self.parser.expect(TokenKind::String)?;
		let string = excerpt_as_string_contents(self.parser.report.clone(), tk_string.excerpt.as_ref().unwrap().as_ref(), &tk_string.span)?;
		
		for mut byte in string.bytes()
		{
			for _ in 0..8
			{
				let bit = byte & 0x80 != 0;
				self.state.output_bit(self.parser.report.clone(), bit, false, &tk_string.span, Some(&tk_string.span))?;
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
	
	
	fn parse_directive_incbin(&mut self) -> Result<(), ()>
	{
		let tk_filename = self.parser.expect(TokenKind::String)?;
		let filename = excerpt_as_string_contents(self.parser.report.clone(), tk_filename.excerpt.as_ref().unwrap().as_ref(), &tk_filename.span)?;
		
		let new_filename = filename_navigate(self.parser.report.clone(), &self.cur_filename, &filename, &tk_filename.span)?;
		
		let bytes = self.fileserver.get_bytes(self.parser.report.clone(), &new_filename, Some(&tk_filename.span))?;
		
		for mut byte in bytes
		{
			for _ in 0..8
			{
				let bit = byte & 0x80 != 0;
				self.state.output_bit(self.parser.report.clone(), bit, false, &tk_filename.span, Some(&tk_filename.span))?;
				byte <<= 1;
			}
		}
		
		Ok(())
	}
	
	
	fn parse_directive_incstr(&mut self, bits_per_char: usize) -> Result<(), ()>
	{
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
				self.state.output_bit(self.parser.report.clone(), bit, false, &tk_filename.span, Some(&tk_filename.span))?;
				digit <<= 1;
			}
		}
		
		Ok(())
	}
	
	
	fn parse_directive_data(&mut self, elem_width: usize, tk_hash: &Token, tk_name: &Token) -> Result<(), ()>
	{
		if elem_width == 0
			{ return Err(self.parser.report.error_span("invalid element width", &tk_name.span)); }
		
		let mut is_first = true;
		
		loop
		{
			let ctx = self.state.get_cur_context();
			let expr = Expression::parse(&mut self.parser)?;
			let span = expr.span();
			let output_span = if is_first
				{ expr.span().join(&tk_hash.span) }
			else
				{ expr.span() };
			
			let parsed_expr = ParsedExpression
			{
				ctx: ctx,
				width: elem_width,
				expr: expr
			};
			
			self.state.parsed_exprs.push(parsed_expr);
			
			self.state.output_zero_bits(self.parser.report.clone(), elem_width, false, &span, Some(&output_span))?;
			
			if self.parser.maybe_expect(TokenKind::Comma).is_none()
				{ break; }
				
			is_first = false;
		}
		
		Ok(())
	}
	
	
	fn parse_label(&mut self) -> Result<(), ()>
	{
		let tk_dot = self.parser.maybe_expect(TokenKind::Dot);
		let is_local = tk_dot.is_some();
		let mut name = if is_local { "." } else { "" }.to_string();
	
		let tk_name = self.parser.expect(TokenKind::Identifier)?;
		name.push_str(&tk_name.excerpt.unwrap());
		
		let ctx = self.state.get_cur_context();
		
		let value = if self.parser.maybe_expect(TokenKind::Equal).is_some()
		{		
			let expr = Expression::parse(&mut self.parser)?;
			let value = self.state.expr_eval(self.parser.report.clone(), &ctx, &expr, &mut ExpressionEvalContext::new())?;
			self.parser.expect_linebreak()?;
			value
		}
		else
		{
			let tk_colon = self.parser.expect(TokenKind::Colon)?;
			
			let mut span_full = tk_name.span.join(&tk_colon.span);
			if let Some(tk_dot) = tk_dot
				{ span_full = span_full.join(&tk_dot.span); }
			
			self.state.check_cpudef_active(self.parser.report.clone(), &span_full)?;
			
			let label_align = self.state.cpudef.as_ref().unwrap().label_align;
			if label_align.is_some()
				{ self.state.output_bits_until_aligned(self.parser.report.clone(), label_align.unwrap(), &span_full, None)?; }
			
			let addr = self.state.get_cur_address(self.parser.report.clone(), &span_full)?;
			self.state.mark_label(addr, &span_full);
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
			args.push(self.state.expr_eval(RcReport::new(), &ctx, expr, &mut ExpressionEvalContext::new()).ok());
		}
		
		// If some argument could not be resolved, save instruction for resolving on the second pass.
		if args.iter().any(|a| a.is_none())
		{
			let rule_index = *instr_match.rule_indices.last().unwrap();
			
			let parsed_instr = ParsedInstruction
			{
				rule_index: rule_index,
				span: instr_span.clone(),
				ctx: ctx,
				exprs: instr_match.exprs,
				args: args
			};
			
			self.state.parsed_instrs.push(parsed_instr);
			
			// Also output zero bits to advance the current address.
			let instr_width =
			{
				let rule = &self.state.cpudef.as_ref().unwrap().rules[rule_index];
				rule.production.width().unwrap()
			};
			
			self.state.output_zero_bits(self.parser.report.clone(), instr_width, false, &instr_span, Some(&instr_span))
		}
		
		// ...or if all arguments could be resolved, output instruction now.
		else
		{
			// Apply cascading to find best suited match.
			let mut best_match = 0;
			while best_match < instr_match.rule_indices.len() - 1
			{
				// Evaluate the instruction's production and check whether it
				// generates an error, in which case, just try the next instruction in the series.
				let rule = &self.state.cpudef.as_ref().unwrap().rules[instr_match.rule_indices[best_match]];

				let report = RcReport::new();
				
				let mut args_eval_ctx = ExpressionEvalContext::new();
				for i in 0..args.len()
				{
					let arg = args[i].clone().unwrap();
					self.state.check_expr_constraint(report.clone(), &arg, &rule.params[i].typ, &Span::new_dummy()).ok();
					args_eval_ctx.set_local(rule.params[i].name.clone(), arg);
				}

				self.state.expr_eval(report.clone(), &ctx, &rule.production, &mut args_eval_ctx).ok();
				
				if !report.has_errors()
					{ break; }
				
				best_match += 1;
			}
			
			let mut parsed_instr = ParsedInstruction
			{
				rule_index: instr_match.rule_indices[best_match],
				span: instr_span,
				ctx: ctx,
				exprs: instr_match.exprs,
				args: args
			};
			
			self.state.output_parsed_instr(self.parser.report.clone(), &mut parsed_instr)
		}
	}
	
	
	fn parse_usize(&mut self) -> Result<usize, ()>
	{
		let ctx = self.state.get_cur_context();
		
		let expr = Expression::parse(&mut self.parser)?;
		
		let value = match self.state.expr_eval(self.parser.report.clone(), &ctx, &expr, &mut ExpressionEvalContext::new())
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