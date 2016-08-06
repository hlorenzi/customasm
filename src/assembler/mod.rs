use definition::Definition;
use util::bitvec::BitVec;
use util::error::Error;
use util::expression::{Expression, ExpressionName};
use util::label::{LabelManager, LabelContext};
use util::misc;
use util::parser::Parser;
use util::tokenizer;
use rule::{Rule, PatternSegment};
use std::path::PathBuf;


struct Assembler<'def>
{
	def: &'def Definition,
	cur_address: usize,
	cur_output: usize,
	labels: LabelManager,
	unresolved_instructions: Vec<Instruction>,
	unresolved_expressions: Vec<UnresolvedExpression>,
	
	output_bits: BitVec
}


struct Instruction
{
	rule_index: usize,
	label_ctx: LabelContext,
	address: usize,
	output: usize,
	arguments: Vec<Expression>
}


struct UnresolvedExpression
{
	expr: Expression,
	label_ctx: LabelContext,
	address: usize,
	output: usize,
	bit_num: usize
}


pub fn assemble(def: &Definition, src_filename: &str, src: &[char]) -> Result<BitVec, Error>
{
	let mut assembler = Assembler
	{
		def: def,
		cur_address: 0,
		cur_output: 0,
		labels: LabelManager::new(),
		unresolved_instructions: Vec::new(),
		unresolved_expressions: Vec::new(),
		output_bits: BitVec::new()
	};
	
	try!(translate_file(&mut assembler, src_filename, src));	
	
	let mut unresolved_insts = Vec::new();
	unresolved_insts.append(&mut assembler.unresolved_instructions);
	
	for inst in unresolved_insts.iter()
	{
		let value = try!(resolve_instruction(&assembler, &inst));
		assembler.output_aligned_at(inst.output, &value);
	}
	
	let mut unresolved_exprs = Vec::new();
	unresolved_exprs.append(&mut assembler.unresolved_expressions);
	
	for unres in unresolved_exprs.iter()
	{
		let value = try!(assembler.resolve_expr(&unres.expr, unres.label_ctx));
		assembler.output_aligned_at(unres.output, &value.slice(unres.bit_num - 1, 0));
	}
	
	Ok(assembler.output_bits)
}


impl<'def> Assembler<'def>
{
	pub fn advance_bits(&mut self, bit_num: usize)
	{
		assert!(bit_num % self.def.align_bits == 0);
		let address_inc = bit_num / self.def.align_bits;
		self.cur_output += address_inc;
		self.cur_address += address_inc;
	}
	

	pub fn output_aligned(&mut self, bitvec: &BitVec)
	{
		let aligned_index = self.cur_output * self.def.align_bits;
		self.output_bits.set(aligned_index, bitvec);
		self.advance_bits(bitvec.len());
	}
	
	
	pub fn output_aligned_at(&mut self, index: usize, bitvec: &BitVec)
	{
		let aligned_index = index * self.def.align_bits;
		self.output_bits.set(aligned_index, bitvec);
	}
	
	
	pub fn can_resolve_expr(&self, expr: &Expression, ctx: LabelContext) -> Result<bool, Error>
	{
		expr.can_resolve(&|expr, _|
		{
			match expr
			{
				ExpressionName::GlobalVariable(name) => Ok(self.labels.does_global_exist(name)),
				ExpressionName::LocalVariable(name) => Ok(self.labels.does_local_exist(ctx, name))
			}
		})
	}
	
	
	pub fn get_expr_minimum_bit_num(&self, expr: &Expression, ctx: LabelContext) -> Result<usize, Error>
	{
		expr.get_minimum_bit_num(&|expr, _|
		{
			let maybe_bitvec = match expr
			{
				ExpressionName::GlobalVariable(name) => self.labels.get_global_value(name),
				ExpressionName::LocalVariable(name) => self.labels.get_local_value(ctx, name)
			};
			
			match maybe_bitvec
			{
				Some(bitvec) => Ok(bitvec.len()),
				None => Ok(self.def.address_bits)
			}
		})
	}
	
	
	pub fn resolve_production(&self, rule: &Rule, inst: &Instruction, expr: &Expression) -> Result<BitVec, Error>
	{
		expr.resolve(&|expr, _|
		{
			let argument = match expr
			{
				ExpressionName::GlobalVariable(name) => &inst.arguments[rule.get_argument(name).unwrap()],
				ExpressionName::LocalVariable(_) => panic!("local variable in production; invalid definition")
			};
			
			self.resolve_expr(argument, inst.label_ctx)
		})
	}
	
	
	pub fn resolve_expr(&self, expr: &Expression, ctx: LabelContext) -> Result<BitVec, Error>
	{
		expr.resolve(&|expr, span|
		{
			let maybe_bitvec = match expr
			{
				ExpressionName::GlobalVariable(name) => self.labels.get_global_value(name),
				ExpressionName::LocalVariable(name) => self.labels.get_local_value(ctx, name)
			};
			
			match maybe_bitvec
			{
				Some(bitvec) => Ok(bitvec.clone()),
				None =>
				{
					match expr
					{
						ExpressionName::GlobalVariable(name) =>
							Err(Error::new_with_span(format!("unknown `{}`", name), span.clone())),
						ExpressionName::LocalVariable(name) =>
							Err(Error::new_with_span(format!("unknown local `{}`", name), span.clone())),
					}
				}
			}
		})
	}
}


fn translate_file(assembler: &mut Assembler, src_filename: &str, src: &[char]) -> Result<(), Error>
{
	let tokens = tokenizer::tokenize(src_filename, src);
	let mut parser = Parser::new(src_filename, &tokens);
	
	while !parser.is_over()
	{
		if parser.current().is_operator(".")
			{ try!(translate_directive(assembler, &mut parser)); }
		else if parser.current().is_identifier() && parser.next(1).is_operator(":")
			{ try!(translate_global_label(assembler, &mut parser)); }
		else if parser.current().is_operator("'") && parser.next(1).is_identifier() && parser.next(2).is_operator(":")
			{ try!(translate_local_label(assembler, &mut parser)); }
		else
			{ try!(translate_instruction(assembler, &mut parser)); }
	}
	
	Ok(())
}


fn translate_directive(assembler: &mut Assembler, parser: &mut Parser) -> Result<(), Error>
{
	try!(parser.expect_operator("."));
	let directive_token = try!(parser.expect_identifier()).clone();
	let directive = directive_token.identifier();
	
	if directive.chars().next() == Some('d')
	{
		let mut bit_num_str = directive.clone();
		bit_num_str.remove(0);
		
		match usize::from_str_radix(&bit_num_str, 10)
		{
			Ok(bit_num) =>
			{
				if bit_num % assembler.def.align_bits != 0
					{ return Err(parser.make_error(format!("literal is not aligned to `{}` bits", assembler.def.align_bits), &directive_token.span)); }
			
				return translate_literal(assembler, parser, bit_num);
			}
			Err(..) => { }
		}
	}
	
	match directive.as_ref()
	{
		"address" => assembler.cur_address = try!(parser.expect_number()).number_usize(),
		"output" => assembler.cur_output = try!(parser.expect_number()).number_usize(),
		"include" =>
		{
			let include_filename = try!(parser.expect_string()).string().clone();
			let mut cur_path = PathBuf::from(parser.get_filename());
			cur_path.set_file_name(&include_filename);
			let include_chars = misc::read_file(&cur_path);
			try!(translate_file(assembler, &cur_path.to_string_lossy().into_owned(), &include_chars));
		}
		"includebin" => 
		{
			let include_filename = try!(parser.expect_string()).string().clone();
			let mut cur_path = PathBuf::from(parser.get_filename());
			cur_path.set_file_name(&include_filename);
			let include_bitvec = BitVec::new_from_bytes(&misc::read_file_bytes(&cur_path));
			assembler.output_aligned(&include_bitvec);
		}
		_ => return Err(parser.make_error(format!("unknown directive `{}`", directive), &directive_token.span))
	}
	
	try!(parser.expect_separator_linebreak());
	Ok(())
}


fn translate_literal(assembler: &mut Assembler, parser: &mut Parser, bit_num: usize) -> Result<(), Error>
{
	loop
	{
		let expr = try!(Expression::new_by_parsing(parser));
		
		if try!(assembler.can_resolve_expr(&expr, assembler.labels.get_cur_context()))
		{
			let bits = try!(assembler.resolve_expr(&expr, assembler.labels.get_cur_context()));			
			assembler.output_aligned(&bits.slice(bit_num - 1, 0));
		}
		else
		{
			assembler.unresolved_expressions.push(UnresolvedExpression
			{
				expr: expr,
				label_ctx: assembler.labels.get_cur_context(),
				address: assembler.cur_address,
				output: assembler.cur_output,
				bit_num: bit_num
			});
			
			assembler.cur_address += bit_num / assembler.def.align_bits;
			assembler.cur_output += bit_num / assembler.def.align_bits;
		}
		
		if !parser.match_operator(",")
			{ break; }
	}
	
	try!(parser.expect_separator_linebreak());
	Ok(())
}


fn translate_global_label(assembler: &mut Assembler, parser: &mut Parser) -> Result<(), Error>
{
	let label_token = try!(parser.expect_identifier()).clone();
	let label = label_token.identifier();
	try!(parser.expect_operator(":"));
	
	if assembler.labels.does_global_exist(label)
		{ return Err(parser.make_error(format!("duplicate global label `{}`", label), &label_token.span)); }
	
	assembler.labels.add_global(
		label.clone(),
		BitVec::new_from_usize(assembler.cur_address));
	
	try!(parser.expect_separator_linebreak());
	Ok(())
}


fn translate_local_label(assembler: &mut Assembler, parser: &mut Parser) -> Result<(), Error>
{
	try!(parser.expect_operator("'"));
	let label_token = try!(parser.expect_identifier()).clone();
	let label = label_token.identifier();
	try!(parser.expect_operator(":"));
	
	if assembler.labels.does_local_exist(assembler.labels.get_cur_context(), label)
		{ return Err(parser.make_error(format!("duplicate local label `{}`", label), &label_token.span)); }
	
	let local_ctx = assembler.labels.get_cur_context();
	assembler.labels.add_local(
		local_ctx,
		label.clone(),
		BitVec::new_from_usize(assembler.cur_address));
	
	try!(parser.expect_separator_linebreak());
	Ok(())
}


fn translate_instruction<'p, 'f, 'tok>(assembler: &mut Assembler, parser: &'p mut Parser<'f, 'tok>) -> Result<(), Error>
{
	let mut maybe_inst = None;
	let inst_span = parser.current().span.clone();

	for rule_index in 0..assembler.def.rules.len()
	{
		let mut rule_parser = parser.clone_from_current();
		
		match try!(try_match_rule(assembler, &mut rule_parser, rule_index))
		{
			Some(inst) =>
			{
				if try!(can_resolve_instruction(assembler, &inst))
				{
					maybe_inst = Some(inst);
					*parser = rule_parser;
					break;
				}
				else
				{
					maybe_inst = Some(inst);
					*parser = rule_parser;
				}
			}
			None => { }
		}
	}
	
	match maybe_inst
	{
		Some(inst) =>
		{
			let rule = &assembler.def.rules[inst.rule_index];
			assembler.advance_bits(rule.production_bit_num);
			
			if try!(can_resolve_instruction(assembler, &inst))
			{
				let value = try!(resolve_instruction(assembler, &inst));
				assembler.output_aligned_at(inst.output, &value);
			}
			else
				{ assembler.unresolved_instructions.push(inst); }
		}
	
		None => return Err(parser.make_error("no match found for instruction", &inst_span))
	}
	
	try!(parser.expect_separator_linebreak());
	Ok(())
}


fn try_match_rule(assembler: &mut Assembler, parser: &mut Parser, rule_index: usize) -> Result<Option<Instruction>, Error>
{
	let rule = &assembler.def.rules[rule_index];
	
	let mut inst = Instruction
	{
		label_ctx: assembler.labels.get_cur_context(),
		rule_index: rule_index,
		address: assembler.cur_address,
		output: assembler.cur_output,
		arguments: Vec::new()
	};
	
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
			
			&PatternSegment::Argument(arg_index) =>
			{
				let typ = rule.get_argument_type(arg_index);
				
				match Expression::new_by_parsing(parser)
				{
					Ok(expr) =>
					{
						let expr_len = try!(assembler.get_expr_minimum_bit_num(&expr, assembler.labels.get_cur_context()));
						if expr_len <= typ.bit_num
							{ inst.arguments.push(expr); }
						else
							{ return Ok(None); }
					}
					Err(..) => return Ok(None)
				};
			}
		}
	}
	
	Ok(Some(inst))
}


fn can_resolve_instruction(assembler: &Assembler, inst: &Instruction) -> Result<bool, Error>
{
	for expr in inst.arguments.iter()
	{
		if !(try!(assembler.can_resolve_expr(expr, inst.label_ctx)))
			{ return Ok(false); }
	}
	
	Ok(true)
}


fn resolve_instruction(assembler: &Assembler, inst: &Instruction) -> Result<BitVec, Error>
{
	let mut bitvec = BitVec::new();
	let rule = &assembler.def.rules[inst.rule_index];
	
	for expr in rule.production_segments.iter()
		{ bitvec.push(&try!(assembler.resolve_production(rule, inst, expr))); }
	
	Ok(bitvec)
}