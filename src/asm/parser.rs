use diagn::Message;
use syntax::{TokenKind, tokenize, Parser};
use expr::ExpressionValue;
use asm::{AssemblerState, ParsedInstruction, ExpressionContext};


pub struct AssemblerParser<'a, 'b>
where 'b: 'a
{
	pub state: &'a mut AssemblerState<'b>,
	pub parser: Parser<'a>
}
	
	
impl<'a, 'b> AssemblerParser<'a, 'b>
{
	pub fn parse_file<S>(state: &mut AssemblerState, filename: S) -> Result<(), Message>
	where S: Into<String>
	{
		let filename_owned = filename.into();
		let chars = state.fileserver.get_chars(&filename_owned)?;
		let tokens = tokenize(state.reporter, filename_owned, &chars);
		
		let mut parser = AssemblerParser
		{
			state: state,
			parser: Parser::new(&tokens)
		};
		
		parser.parse()
	}
	
	
	fn parse(&mut self) -> Result<(), Message>
	{
		while !self.parser.is_over()
		{
			self.parse_line()?;
			self.parser.expect_linebreak()?;
		}
		
		Ok(())
	}
	
	
	fn parse_line(&mut self) -> Result<(), Message>
	{
		self.parse_instruction()
	}
	
	
	fn parse_instruction(&mut self) -> Result<(), Message>
	{
		let instr_span_start = self.parser.next().span;
		
		// Find matching rule patterns.
		let (instr_match, new_parser) = match self.state.pattern_matcher.parse_match(self.parser.clone())
		{
			Some((instr_match, new_parser)) => (instr_match, new_parser),
			None =>
			{
				self.parser.skip_until_linebreak();
				let instr_span = instr_span_start.join(&self.parser.prev().span);
				
				self.state.reporter.message(Message::error_span("no match for instruction found", &instr_span));
				return Ok(());
			}
		};
		
		self.parser = new_parser;
		let instr_span = instr_span_start.join(&self.parser.prev().span);
		
		let ctx = ExpressionContext
		{
			label_ctx: self.state.labels.get_cur_context(),
			address: self.state.cur_address,
			writehead: self.state.cur_writehead
		};
		
		// Resolve as many arguments as possible right now.
		let mut args: Vec<Option<ExpressionValue>> = Vec::new();
		
		for expr in &instr_match.exprs
			{ args.push(self.state.expr_eval(&ctx, expr).ok()); }
		
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
				if self.state.rule_check_all_constraints_satisfied(rule, &get_arg, &ctx, &instr_span).ok().is_some()
					{ break; }
					
				best_match += 1;
			}
			
			best_match
		};
		
		let rule = &self.state.instrset.rules[instr_match.rule_indices[best_match]];
		
		// Having found the best matching rule, save it to be output on the second pass.
		// Remaining argument resolution and constraint checking will be done then.
		let parsed_instr = ParsedInstruction
		{
			rule_index: instr_match.rule_indices[best_match],
			span: instr_span,
			ctx: ctx,
			exprs: instr_match.exprs,
			args: args
		};
		
		self.state.parsed_instrs.push(parsed_instr);
		
		let instr_width = rule.production_width();
		self.state.cur_address += instr_width;
		self.state.cur_writehead += instr_width;
		
		Ok(())
	}
}