use parser::{Parser, ParserError};
use tokenizer;
use tokenizer::Span;
use bitvec::BitVec;
use definition::{Definition};
use rule::{PatternSegment, ProductionSegment};
use std::collections::HashMap;


struct Translator<'def>
{
	def: &'def Definition,
	cur_address: usize,
	cur_output: usize,
	labels: HashMap<String, usize>,
	unresolved_instructions: Vec<Instruction>,
	
	output_bits: BitVec
}


struct Instruction
{
	span: Span,
	rule_index: usize,
	address: usize,
	output: usize,
	arguments: Vec<Expression>
}


enum Expression
{
	LiteralUInt(BitVec),
	Variable(String)
}


pub fn translate(def: &Definition, src: &[char]) -> Result<BitVec, ParserError>
{
	let mut translator = Translator
	{
		def: def,
		cur_address: 0,
		cur_output: 0,
		labels: HashMap::new(),
		unresolved_instructions: Vec::new(),
		output_bits: BitVec::new()
	};
	
	let tokens = tokenizer::tokenize(src);
	let mut parser = Parser::new(&tokens);
	
	while !parser.is_over()
	{
		if parser.current().is_operator(".")
			{ try!(translate_directive(&mut translator, &mut parser)); }
		else if parser.current().is_identifier() && parser.next(1).is_operator(":")
			{ try!(translate_label(&mut translator, &mut parser)); }
		else
			{ try!(translate_instruction(&mut translator, &mut parser)); }
	}
	
	let mut unresolved_insts = Vec::new();
	unresolved_insts.append(&mut translator.unresolved_instructions);
	
	for inst in unresolved_insts.iter()
	{
		match resolve_instruction(&translator, &inst)
		{
			Ok(bits) =>
			{
				let cur_output = inst.output * translator.def.align_bits;
				translator.output(cur_output, &bits);
			}
			
			Err(msg) => return Err(ParserError::new(msg, inst.span))
		}
	}
	
	Ok(translator.output_bits)
}


impl<'def> Translator<'def>
{
	pub fn output(&mut self, index: usize, bitvec: &BitVec)
	{
		self.output_bits.set(index, bitvec);
	}
}


fn translate_directive(translator: &mut Translator, parser: &mut Parser) -> Result<(), ParserError>
{
	try!(parser.expect_operator("."));
	let directive_token = try!(parser.expect_identifier()).clone();
	let directive = directive_token.identifier();
	
	match directive.as_ref()
	{
		"address" => translator.cur_address = try!(parser.expect_number()).number_usize(),
		"output" => translator.cur_output = try!(parser.expect_number()).number_usize(),
		_ => return Err(ParserError::new(format!("unknown directive `{}`", directive), directive_token.span))
	}
	
	Ok(())
}


fn translate_label(translator: &mut Translator, parser: &mut Parser) -> Result<(), ParserError>
{
	let label_token = try!(parser.expect_identifier()).clone();
	let label = label_token.identifier();
	try!(parser.expect_operator(":"));
	
	if translator.labels.contains_key(label)
		{ return Err(ParserError::new(format!("duplicate label `{}`", label), label_token.span)); }
	
	translator.labels.insert(label.clone(), translator.cur_address);
	Ok(())
}


fn translate_instruction<'p, 'tok>(translator: &mut Translator, parser: &'p mut Parser<'tok>) -> Result<(), ParserError>
{
	let mut maybe_inst = None;
	let inst_span = parser.current().span;

	for rule_index in 0..translator.def.rules.len()
	{
		let mut rule_parser = parser.clone_from_current();
		
		match try_match_rule(translator, &mut rule_parser, rule_index)
		{
			Some(inst) =>
			{
				if can_resolve_instruction(translator, &inst)
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
			let rule = &translator.def.rules[inst.rule_index];
			translator.cur_address += rule.production_bit_num / translator.def.align_bits;
			translator.cur_output += rule.production_bit_num / translator.def.align_bits;
			
			if can_resolve_instruction(translator, &inst)
			{
				match resolve_instruction(translator, &inst)
				{
					Ok(bits) =>
					{
						let cur_output = inst.output * translator.def.align_bits;
						translator.output(cur_output, &bits);
					}
					
					Err(msg) => return Err(ParserError::new(msg, inst.span))
				}
			}
			else
			{
				translator.unresolved_instructions.push(inst);
			}
		}
	
		None => return Err(ParserError::new("no match found for instruction".to_string(), inst_span))
	}
	
	Ok(())
}


fn try_match_rule(translator: &mut Translator, parser: &mut Parser, rule_index: usize) -> Option<Instruction>
{
	let rule = &translator.def.rules[rule_index];
	
	let mut inst = Instruction
	{
		span: parser.current().span,
		rule_index: rule_index,
		address: translator.cur_address,
		output: translator.cur_output,
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
					{ return None; }
			}
			
			&PatternSegment::Argument(arg_index) =>
			{
				let typ = rule.get_argument_type(arg_index);
				
				match parse_expression(parser)
				{
					Ok(expr) =>
					{
						let expr_len = get_expression_min_bit_num(translator, &expr);
						if expr_len <= typ.bit_num
							{ inst.arguments.push(expr); }
						else
							{ return None; }
					}
					Err(..) => return None
				};
			}
		}
	}
	
	Some(inst)
}


fn parse_expression(parser: &mut Parser) -> Result<Expression, ParserError>
{
	if parser.current().is_identifier()
		{ Ok(Expression::Variable(try!(parser.expect_identifier()).identifier().clone())) }
	
	else if parser.current().is_number()
	{
		let token = try!(parser.expect_number());
		let (radix, value_str) = token.number();
		
		match BitVec::new_from_str(radix, value_str)
		{
			Err(msg) => Err(ParserError::new(msg, token.span)),
			Ok(bitvec) => Ok(Expression::LiteralUInt(bitvec))
		}
	}
	
	else
		{ Err(ParserError::new("expected expression".to_string(), parser.current().span)) }
}


fn can_resolve_instruction(translator: &Translator, inst: &Instruction) -> bool
{
	for segment in translator.def.rules[inst.rule_index].production_segments.iter()
	{
		match segment
		{
			&ProductionSegment::Literal(..) => { }
			
			&ProductionSegment::Argument { index, leftmost_bit: _, rightmost_bit: _ } =>
			{
				if !can_resolve_expression(translator, &inst.arguments[index])
					{ return false; }
			}
		}
	}
	
	true
}


fn resolve_instruction(translator: &Translator, inst: &Instruction) -> Result<BitVec, String>
{
	let mut bitvec = BitVec::new();
	
	for segment in translator.def.rules[inst.rule_index].production_segments.iter()
	{
		match segment
		{
			&ProductionSegment::Literal(ref literal_bitvec) =>
			{
				bitvec.push(&literal_bitvec);
			}
			
			&ProductionSegment::Argument { index, leftmost_bit, rightmost_bit } =>
			{
				let expr_bitvec = try!(resolve_expression(translator, &inst.arguments[index], rightmost_bit - leftmost_bit));
				bitvec.push(&expr_bitvec);
			}
		}
	}
	
	Ok(bitvec)
}


fn can_resolve_expression(translator: &Translator, expr: &Expression) -> bool
{
	match expr
	{
		&Expression::LiteralUInt(..) =>
			{ return true; }
		
		&Expression::Variable(ref name) =>
			{ return translator.labels.contains_key(name); }
	}
}


fn get_expression_min_bit_num(translator: &Translator, expr: &Expression) -> usize
{
	match expr
	{
		&Expression::LiteralUInt(ref literal_bitvec) => return literal_bitvec.len(),
		&Expression::Variable(ref name) =>
		{
			if !translator.labels.contains_key(name)
				{ translator.def.address_bits }
			else
			{
				let bitvec = BitVec::new_from_usize(translator.labels[name]);
				bitvec.len()
			}
		}
	}
}


fn resolve_expression(translator: &Translator, expr: &Expression, bit_num: usize) -> Result<BitVec, String>
{
	match expr
	{
		&Expression::LiteralUInt(ref literal_bitvec) =>
		{
			let mut bitvec = literal_bitvec.clone();
			
			if bitvec.len() > bit_num
				{ return Err("argument does not fit".to_string()); }
			
			bitvec.zero_extend(bit_num);
			return Ok(bitvec);
		}
		
		&Expression::Variable(ref name) =>
		{
			if !translator.labels.contains_key(name)
				{ return Err(format!("unknown variable `{}`", name)); }
				
			let mut bitvec = BitVec::new_from_usize(translator.labels[name]);
			bitvec.zero_extend(bit_num);
			return Ok(bitvec);
		}
	}
}