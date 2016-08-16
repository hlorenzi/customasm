use definition::Definition;
use rule::{Rule, PatternSegment};
use util::bigint::BigInt;
use util::bitvec::BitVec;
use util::error::{Error, handle_opt_span, handle_result_span};
use util::expression::{Expression, ExpressionVariable, ExpressionValue};
use util::filehandler::{FileHandler, CustomFileHandler};
use util::label::{LabelManager, LabelContext};
use util::parser::Parser;
use util::tokenizer;
use util::tokenizer::Span;
use std::path::{Path, PathBuf};


/// Holds intermediate information during assembly.
struct Assembler<'a>
{
	def: &'a Definition,
	filehandler: &'a FileHandler,
	
	cur_address: usize,
	cur_output: usize,
	labels: LabelManager,
	unresolved_instructions: Vec<Instruction>,
	unresolved_expressions: Vec<UnresolvedExpression>,
	
	output_bits: BitVec
}


/// Represents a parsed instruction with a matched rule.
/// Includes the context in which it appeared in the
/// source-code. Also includes full argument expressions
/// as seen in the source-code.
struct Instruction
{
	rule_index: usize,
	label_ctx: LabelContext,
	address: usize,
	output: usize,
	arguments: Vec<Expression>
}


/// Represents an unresolved expression in a data
/// directive. Includes the context in which it 
/// appeared in the source-code.
struct UnresolvedExpression
{
	expr: Expression,
	label_ctx: LabelContext,
	address: usize,
	output: usize,
	data_width: usize
}


/// Convenience interface to the assembly process.
pub fn assemble_single(def: &Definition, src: &str) -> Result<BitVec, Error>
{
	let mut filehandler = CustomFileHandler::new();
	filehandler.add("main", src);
	
	assemble(def, &filehandler, &PathBuf::from("main"))
}



/// Main interface to the assembly process.
pub fn assemble(def: &Definition, filehandler: &FileHandler, main_filename: &Path) -> Result<BitVec, Error>
{
	// Prepare an assembler state.
	let mut assembler = Assembler
	{
		def: def,
		filehandler: filehandler,
		
		cur_address: 0,
		cur_output: 0,
		labels: LabelManager::new(),
		unresolved_instructions: Vec::new(),
		unresolved_expressions: Vec::new(),
		
		output_bits: BitVec::new()
	};
	
	
	// == First-pass ==
	
	// Parse the main file.
	try!(assembler.parse_file(main_filename));	
	
	
	// == Second-pass ==
	
	// Resolve remaining instructions.
	let instrs: Vec<_> = assembler.unresolved_instructions.drain(..).collect();
	for instr in instrs
	{
		try!(assembler.resolve_instruction(&instr));
	}
	
	// Resolve remaining expressions in literals.
	let exprs: Vec<_> = assembler.unresolved_expressions.drain(..).collect();
	for expr in exprs
	{
		match try!(assembler.resolve_expr(&expr.expr, expr.label_ctx, expr.address))
		{
			ExpressionValue::Integer(ref bigint) =>
				assembler.output_integer_at(expr.output, expr.data_width, &bigint),
				
			_ => return Err(Error::new_with_span("invalid expression", expr.expr.span.clone()))
		}
	}
	
	// Return output bits.
	Ok(assembler.output_bits)
}


impl<'def> Assembler<'def>
{
	/// Main parsing function.
	/// Reads source-code lines and decides how to decode them.
	fn parse_file(&mut self, filename: &Path) -> Result<(), Error>
	{
		let chars = try!(self.filehandler.read_chars(filename));
		let tokens = tokenizer::tokenize(filename.to_string_lossy().into_owned(), &chars);
		let mut parser = Parser::new(&tokens);
		
		while !parser.is_over()
		{
			if parser.current().is_operator(".")
				{ try!(self.parse_directive(&mut parser, filename)); }
				
			else if parser.current().is_identifier() && parser.next(1).is_operator("=")
				{ try!(self.parse_global_constant(&mut parser)); }
				
			else if parser.current().is_identifier() && parser.next(1).is_operator(":")
				{ try!(self.parse_global_label(&mut parser)); }
				
			else if parser.current().is_operator("'") && parser.next(1).is_identifier() && parser.next(2).is_operator(":")
				{ try!(self.parse_local_label(&mut parser)); }
				
			else
				{ try!(self.parse_instruction(&mut parser)); }
		}
		
		Ok(())
	}


	fn parse_directive(&mut self, parser: &mut Parser, cur_path: &Path) -> Result<(), Error>
	{
		try!(parser.expect_operator("."));
		let (directive, directive_span) = try!(parser.expect_identifier());
		
		// If the directive starts with a 'd', it might
		// be a data directive.
		if directive.chars().next() == Some('d')
		{
			// Try to parse a number after the 'd'.
			match usize::from_str_radix(&directive[1..], 10)
			{
				Ok(data_width) =>
				{
					// If there was a valid number after the 'd',
					// check for validity, and then
					// call a more specialized function.
					if data_width % self.def.align_bits != 0
					{
						return Err(Error::new_with_span(
							format!("data directive is not aligned to `{}` bits", self.def.align_bits),
							directive_span));
					}
				
					if data_width > 63
					{
						return Err(Error::new_with_span(
							"data directive bit width is currently not supported",
							directive_span));
					}
					
					return self.parse_data_directive(parser, data_width);
				}
				
				Err(_) =>
				{
					// If there was an invalid number after the 'd',
					// fallthrough to the directive-matcher below.
				}
			}
		}
		
		// Parse text-only directives.
		match directive.as_ref()
		{
			"address" =>
				self.cur_address = try!(self.parse_integer(parser)),
			
			"output" => 
				self.cur_output = try!(self.parse_integer(parser)),
			
			"res" => 
			{
				let bits = self.def.align_bits * try!(self.parse_integer(parser));
				self.advance_address(bits);
			}
			
			"include" =>
			{
				let (new_path, span) = try!(self.parse_relative_filename(parser, cur_path));
				
				try!(handle_result_span(
					self.parse_file(&new_path), &span));
			}
			
			"includebin" => 
			{
				let (new_path, span) = try!(self.parse_relative_filename(parser, cur_path));
				
				let bytes = try!(handle_result_span(
					self.filehandler.read_bytes(&new_path), &span));
					
				let bitvec = BitVec::new_from_bytes(&bytes);
				
				if bitvec.len() % self.def.align_bits != 0
				{
					return Err(Error::new_with_span(
						format!("included file size is not aligned to `{}` bits", self.def.align_bits), span));
				}
				
				self.output_bitvec(&bitvec);
			}
			
			_ => return Err(Error::new_with_span(format!("unknown directive `{}`", directive), directive_span))
		}
		
		try!(parser.expect_linebreak_or_end());
		Ok(())
	}


	fn parse_data_directive(&mut self, parser: &mut Parser, data_size: usize) -> Result<(), Error>
	{
		// Parse expressions until there isn't a comma.
		loop
		{
			let expr = try!(Expression::new_by_parsing(parser));
			
			try!(self.output_expression(expr, data_size));
			
			if !parser.match_operator(",")
				{ break; }
		}
		
		try!(parser.expect_linebreak_or_end());
		Ok(())
	}


	fn parse_global_constant(&mut self, parser: &mut Parser) -> Result<(), Error>
	{
		let (label, label_span) = try!(parser.expect_identifier());
		try!(parser.expect_operator("="));
		
		// Check for duplicate global labels.
		if self.labels.does_global_exist(&label)
			{ return Err(Error::new_with_span(format!("duplicate global label `{}`", label), label_span)); }
		
		// Resolve constant value.
		let expr = try!(Expression::new_by_parsing(parser));
		let value = try!(self.resolve_expr_current(&expr));
		
		// Store it.
		self.labels.add_global(label, value);
		
		try!(parser.expect_linebreak_or_end());
		Ok(())
	}


	fn parse_global_label(&mut self, parser: &mut Parser) -> Result<(), Error>
	{
		let (label, label_span) = try!(parser.expect_identifier());
		try!(parser.expect_operator(":"));
		
		// Check for duplicate global labels.
		if self.labels.does_global_exist(&label)
			{ return Err(Error::new_with_span(format!("duplicate global label `{}`", label), label_span)); }
		
		// Store as current address.
		self.labels.add_global(
			label,
			ExpressionValue::Integer(BigInt::from_usize(self.cur_address)));
		
		try!(parser.expect_linebreak_or_end());
		Ok(())
	}


	fn parse_local_label(&mut self, parser: &mut Parser) -> Result<(), Error>
	{
		try!(parser.expect_operator("'"));
		let (label, label_span) = try!(parser.expect_identifier());
		try!(parser.expect_operator(":"));
		
		let local_ctx = self.labels.get_cur_context();
		
		// Check for duplicate local labels within the same context.
		if self.labels.does_local_exist(local_ctx, &label)
			{ return Err(Error::new_with_span(format!("duplicate local label `{}`", label), label_span)); }
		
		// Store as current address.
		self.labels.add_local(
			local_ctx,
			label,
			ExpressionValue::Integer(BigInt::from_usize(self.cur_address)));
		
		try!(parser.expect_linebreak_or_end());
		Ok(())
	}


	fn parse_instruction<'p, 'tok>(&mut self, parser: &'p mut Parser<'tok>) -> Result<(), Error>
	{
		let mut maybe_match = None;
		let instr_span = parser.current().span.clone();

		// Try every rule from the definition.
		for rule_index in 0..self.def.rules.len()
		{
			// Clone the parser, to maintain the current one stationary.
			// If the rule doesn't match, the clone is simply discarded.
			// If it does match, the clone will become the main parser.
			let mut rule_parser = parser.clone_from_current();
			
			match try!(self.try_match_rule(&mut rule_parser, rule_index))
			{
				Some(instr) =>
				{
					let can_resolve = try!(self.can_resolve_instruction(&instr));
					
					maybe_match = Some((instr, rule_parser));
					
					if can_resolve
						{ break; }
				}
				
				None =>
				{
					// If the rule didn't match, just continue trying
					// with the next rule.
				}
			}
		}
		
		// Check whether there was a rule match.
		match maybe_match
		{
			Some((instr, new_parser)) =>
			{
				*parser = new_parser;
				try!(self.output_instruction(instr));
			}
			
			None => return Err(Error::new_with_span("no match found for instruction", instr_span))
		}
		
		try!(parser.expect_linebreak_or_end());
		Ok(())
	}


	fn try_match_rule(&mut self, parser: &mut Parser, rule_index: usize) -> Result<Option<Instruction>, Error>
	{
		let rule = &self.def.rules[rule_index];
		
		let mut instr = Instruction
		{
			label_ctx: self.labels.get_cur_context(),
			rule_index: rule_index,
			address: self.cur_address,
			output: self.cur_output,
			arguments: Vec::new()
		};
		
		// Try matching against every segment in the rule pattern.
		for segment in rule.pattern_segments.iter()
		{
			match segment
			{
				&PatternSegment::Exact(ref chars) =>
				{
					if parser.current().is_identifier() && parser.current().identifier() == chars
						{ parser.advance(); }
						
					else if parser.current().is_operator(&chars)
						{ parser.advance(); }
						
					else
						{ return Ok(None); }
				}
				
				&PatternSegment::Parameter(param_index) =>
				{
					let expr = try!(Expression::new_by_parsing(parser));
					
					if !rule.get_parameter_allow_unresolved(param_index)
					{
						let label_ctx = self.labels.get_cur_context();
						
						if !try!(self.can_resolve_expr(&expr, label_ctx))
							{ return Ok(None); }
						
						match rule.get_parameter_constraint(param_index)
						{
							&None => { },
							
							&Some(ref constraint) =>
							{
								let value = try!(self.resolve_expr(&expr, label_ctx, self.cur_address));
								
								if !try!(self.check_constraint(&constraint, &value, self.cur_address))
									{ return Ok(None); }
							}
						}
					}
					
					instr.arguments.push(expr);
				}
			}
		}
		
		Ok(Some(instr))
	}


	fn advance_address(&mut self, bit_num: usize)
	{
		assert!(bit_num % self.def.align_bits == 0);
		let address_inc = bit_num / self.def.align_bits;
		self.cur_output += address_inc;
		self.cur_address += address_inc;
	}
	
	
	fn output_bitvec(&mut self, bitvec: &BitVec)
	{
		let aligned_index = self.cur_output * self.def.align_bits;
		self.output_bits.set_bitvec(aligned_index, bitvec);
		self.advance_address(bitvec.len());
	}
	

	fn output_integer(&mut self, width: usize, value: &BigInt)
	{
		let aligned_index = self.cur_output * self.def.align_bits;
		self.output_bits.set(aligned_index, width, value);
		self.advance_address(width);
	}
	
	
	fn output_integer_at(&mut self, index: usize, width: usize, value: &BigInt)
	{
		let aligned_index = index * self.def.align_bits;
		self.output_bits.set(aligned_index, width, value);
	}
	
	
	fn output_expression(&mut self, expr: Expression, data_width: usize) -> Result<(), Error>
	{
		let label_ctx = self.labels.get_cur_context();
		
		// Try resolving the expression immediately.
		if try!(self.can_resolve_expr(&expr, label_ctx))
		{
			match try!(self.resolve_expr(&expr, label_ctx, self.cur_address))		
			{
				ExpressionValue::Integer(bigint) =>
					self.output_integer(data_width, &bigint),
				
				_ => return Err(Error::new_with_span("invalid expression type", expr.span.clone()))
			}
		}
		
		// If unresolvable now, store it to be resolved
		// on the second-pass.
		else
		{
			self.unresolved_expressions.push(UnresolvedExpression
			{
				expr: expr,
				label_ctx: label_ctx,
				address: self.cur_address,
				output: self.cur_output,
				data_width: data_width
			});
			
			self.advance_address(data_width);
		}
		
		Ok(())
	}
	
	
	fn output_instruction(&mut self, instr: Instruction) -> Result<(), Error>
	{
		let rule = &self.def.rules[instr.rule_index];
		
		self.advance_address(rule.production_bit_num);
		
		// Try resolving the instruction's arguments immediately.
		if try!(self.can_resolve_instruction(&instr))
			{ try!(self.resolve_instruction(&instr)); }
		
		// If unresolvable now, store it to be resolved
		// on the second-pass.
		else
			{ self.unresolved_instructions.push(instr); }
			
		Ok(())
	}
	
		
	fn parse_integer(&self, parser: &mut Parser) -> Result<usize, Error>
	{
		let expr = try!(Expression::new_by_parsing(parser));
		let value = try!(self.resolve_expr_current(&expr));
		
		let bigint = try!(handle_opt_span(
			value.as_integer(), "expected integer", &expr.span));
			
		handle_opt_span(
			bigint.to_usize(), "invalid value", &expr.span)
	}
	
		
	fn parse_relative_filename(&self, parser: &mut Parser, cur_path: &Path) -> Result<(PathBuf, Span), Error>
	{
		let (filename, span) = try!(parser.expect_string());
		
		let mut new_path = PathBuf::from(cur_path);
		new_path.set_file_name(&filename);
		
		Ok((new_path, span))
	}


	fn can_resolve_instruction(&self, instr: &Instruction) -> Result<bool, Error>
	{
		for expr in instr.arguments.iter()
		{
			if !try!(self.can_resolve_expr(expr, instr.label_ctx))
				{ return Ok(false); }
		}
		
		Ok(true)
	}


	fn resolve_instruction(&mut self, instr: &Instruction) -> Result<(), Error>
	{
		let rule = &self.def.rules[instr.rule_index];
		
		let mut cur_output = instr.output;
		for expr in rule.production_segments.iter()
		{
			match try!(self.resolve_production(rule, instr, expr))
			{
				ExpressionValue::Integer(integer) =>
				{
					let width = expr.get_explicit_bit_num().unwrap();
					assert!(width % self.def.align_bits == 0);
					
					self.output_integer_at(cur_output, width, &integer);
					
					cur_output += width / self.def.align_bits;
				}
				
				_ => return Err(Error::new_with_span("invalid production expression type", expr.span.clone()))
			}
		}
		
		Ok(())
	}
	
	
	fn can_resolve_expr(&self, expr: &Expression, ctx: LabelContext) -> Result<bool, Error>
	{
		expr.can_resolve(&|expr_name, _|
		{
			match expr_name
			{
				&ExpressionVariable::Global(ref name) => Ok(name == "pc" || self.labels.does_global_exist(name)),
				&ExpressionVariable::Local(ref name)  => Ok(self.labels.does_local_exist(ctx, name))
			}
		})
	}
	
	
	fn resolve_expr(&self, expr: &Expression, ctx: LabelContext, pc: usize) -> Result<ExpressionValue, Error>
	{
		expr.resolve(&|name_kind, name_span|
		{
			match name_kind
			{
				&ExpressionVariable::Global(ref name) => match name.as_ref()
				{
					"pc" => Ok(ExpressionValue::Integer(BigInt::from_usize(pc))),
					
					name => match self.labels.get_global(name)
					{
						Some(value) => Ok(value.clone()),
						None => Err(Error::new_with_span(format!("unknown `{}`", name), name_span.clone()))
					}
				},
				
				&ExpressionVariable::Local(ref name) => match self.labels.get_local(ctx, name)
				{
					Some(value) => Ok(value.clone()),
					None => Err(Error::new_with_span(format!("unknown local `{}`", name), name_span.clone()))
				}
			}
		})
	}
	
	
	fn resolve_expr_current(&self, expr: &Expression) -> Result<ExpressionValue, Error>
	{
		let label_ctx = self.labels.get_cur_context();
		let address = self.cur_address;
		
		self.resolve_expr(expr, label_ctx, address)
	}
	
	
	fn resolve_production(&self, rule: &Rule, instr: &Instruction, expr: &Expression) -> Result<ExpressionValue, Error>
	{
		expr.resolve(&|param_kind, _|
		{
			match param_kind
			{
				&ExpressionVariable::Global(ref name) => match name.as_ref()
				{
					"pc" => Ok(ExpressionValue::Integer(BigInt::from_usize(instr.address))),
					
					name =>
					{
						let param_index = rule.get_parameter(name).unwrap();
						let arg_expr = &instr.arguments[param_index];
						
						let arg_value = try!(self.resolve_expr(arg_expr, instr.label_ctx, instr.address));
						
						match rule.get_parameter_constraint(param_index)
						{
							&None => { },
							
							&Some(ref constraint_expr) =>
							{
								if !try!(self.check_constraint(&constraint_expr, &arg_value, instr.address))
									{ return Err(Error::new_with_span("parameter constraint not satisfied", arg_expr.span.clone())); }
							}
						}
						
						Ok(arg_value)
					}
				},
				
				_ => unreachable!()
			}
		})
	}
	
	
	fn check_constraint(&self, constraint: &Expression, argument: &ExpressionValue, pc: usize) -> Result<bool, Error>
	{
		let constraint_result = try!(constraint.resolve(&|expr_name, _|
		{
			match expr_name
			{
				&ExpressionVariable::Global(ref name) => match name.as_ref()
				{
					"_" => Ok(argument.clone()),
					
					"pc" => Ok(ExpressionValue::Integer(BigInt::from_usize(pc))),
					
					_ => unreachable!()
				},
						
				_ => unreachable!()
			}
		}));
		
		match constraint_result
		{
			ExpressionValue::Boolean(b) => Ok(b),					
			_ => Err(Error::new_with_span("invalid constraint expression type", constraint.span.clone()))
		}
	}
}