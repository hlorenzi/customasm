use parser::{Parser, ParserError};
use tokenizer;
use configuration::{Configuration, Rule, PatternSegment};
use numbits;
use std::collections::HashMap;


struct Translator<'cfg>
{
	config: &'cfg Configuration,
	cur_address: usize,
	cur_output: usize,
	labels: HashMap<String, usize>,
	second_pass_rules: Vec<SecondPassRule>,
	
	output_bits: Vec<bool>
}


struct SecondPassRule
{
	rule_index: usize,
	address: usize,
	output: usize
}


pub fn translate(config: &Configuration, src: &[char]) -> Result<Vec<bool>, ParserError>
{
	let mut translator = Translator
	{
		config: config,
		cur_address: 0,
		cur_output: 0,
		labels: HashMap::new(),
		second_pass_rules: Vec::new(),
		output_bits: Vec::new()
	};
	
	let tokens = tokenizer::tokenize(src);
	let mut parser = Parser::new(&tokens);
	
	while !parser.is_over()
	{
		if parser.current().is_operator(".")
			{ try!(translate_directive(&mut translator, &mut parser)); }
		else
			{ try!(translate_instruction(&mut translator, &mut parser)); }
	}
	
	Ok(Vec::new())
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


fn translate_instruction(translator: &mut Translator, parser: &mut Parser) -> Result<(), ParserError>
{
	let mut possible_rules = Vec::new();

	for (index, rule) in translator.config.rules.iter().enumerate()
	{
		match match_rule(translator, rule, &mut parser.clone_from_current())
		{
			Some(token_length) =>
			{
				println!("rule #{} match", index);
				possible_rules.push((index, token_length));
			}
			None => { }
		}
	}
	
	if possible_rules.len() < 1
		{ return Err(ParserError::new("invalid instruction".to_string(), parser.current().span)); }
	
	println!("-- chose rule #{}", possible_rules[0].0);
	
	for _ in 0..possible_rules[0].1
		{ parser.advance(); }
	
	Ok(())
}


fn match_rule(translator: &mut Translator, rule: &Rule, parser: &mut Parser) -> Option<usize>
{
	for (index, segment) in rule.pattern_segments.iter().enumerate()
	{
		match segment
		{
			&PatternSegment::Literal(ref literal) =>
			{
				if parser.current().is_identifier()
				{
					if parser.current().identifier() != literal
						{ return None; }
				}
				else if !parser.current().is_operator(&literal)
					{ return None; }
					
				parser.advance();
			}
			
			&PatternSegment::Variable(ref name, ref typ) =>
			{
				if !(parser.current().is_number())
					{ return None; }
					
				{
					let (radix, value_str) = parser.current().number();
					match numbits::get_min_bit_length(radix, value_str)
					{
						Err(..) => return None,
						Ok(num) =>
						{
							if num > typ.size_bits
								{ return None; }
						}
					};
				}
				
				parser.advance();
			}
		}
	}
	
	Some(parser.current_index())
}