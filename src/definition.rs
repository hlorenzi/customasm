use util::error::Error;
use util::expression::Expression;
use util::parser::Parser;
use util::tokenizer;
use rule::{Rule, PatternSegment, VariableType};


pub struct Definition
{
	pub align_bits: usize,
	pub address_bits: usize,
	pub rules: Vec<Rule>
}


pub fn parse(src_filename: &str, src: &[char]) -> Result<Definition, Error>
{
	let mut def = Definition
	{
		align_bits: 8,
		address_bits: 8,
		rules: Vec::new()
	};
	
	let tokens = tokenizer::tokenize(src_filename, src);
	let mut parser = Parser::new(src_filename, &tokens);
	try!(parse_directives(&mut def, &mut parser));
	try!(parse_rules(&mut def, &mut parser));
	
	Ok(def)
}


fn parse_directives(def: &mut Definition, parser: &mut Parser) -> Result<(), Error>
{
	while parser.match_operator(".")
	{
		let directive = try!(parser.expect_identifier()).clone();
		
		match directive.identifier().as_ref()
		{
			"align" => def.align_bits = try!(parser.expect_number()).number_usize(),
			"address" => def.address_bits = try!(parser.expect_number()).number_usize(),
			_ => return Err(parser.make_error(format!("unknown directive `{}`", directive.identifier()), &directive.span))
		}
		
		try!(parser.expect_separator_linebreak());
	}
	
	Ok(())
}


fn parse_rules(def: &mut Definition, parser: &mut Parser) -> Result<(), Error>
{
	while !parser.is_over()
	{
		let mut rule = Rule::new();
		
		try!(parse_pattern(parser, &mut rule));
		try!(parser.expect_operator("->"));
		try!(parse_production(def, parser, &mut rule));
		
		def.rules.push(rule);
		
		try!(parser.expect_separator_linebreak());
	}
	
	Ok(())
}


fn parse_pattern(parser: &mut Parser, rule: &mut Rule) -> Result<(), Error>
{
	while !parser.current().is_operator("->")
	{
		if parser.current().is_identifier()
		{
			let ident = try!(parser.expect_identifier());
			rule.pattern_segments.push(PatternSegment::Exact(ident.identifier().clone()));
		}
		
		else if parser.match_operator("{")
		{
			let name_token = try!(parser.expect_identifier()).clone();
			let name = name_token.identifier();
			try!(parser.expect_operator(":"));
			let typ = try!(parse_variable_type(parser));
			
			if rule.check_argument_exists(&name)
				{ return Err(parser.make_error(format!("duplicate argument `{}`", name), &name_token.span)); }
			
			let arg_index = rule.add_argument(name.clone(), typ);
			rule.pattern_segments.push(PatternSegment::Argument(arg_index));
			
			try!(parser.expect_operator("}"));
		}
		
		else if parser.current().is_any_operator()
		{
			let op = try!(parser.expect_any_operator());
			rule.pattern_segments.push(PatternSegment::Exact(op.operator().to_string()));
		}
		
		else
			{ return Err(parser.make_error("expected pattern", &parser.current().span)); }
	}
	
	Ok(())
}


fn parse_production(def: &mut Definition, parser: &mut Parser, rule: &mut Rule) -> Result<(), Error>
{
	let begin_span = parser.current().span.clone();
	
	while !parser.current().is_linebreak_or_end()
	{
		let expr = try!(Expression::new_by_parsing_checked(parser, &|name| rule.check_argument_exists(name)));
		
		rule.production_bit_num += match expr.get_explicit_bit_num()
		{
			Some(bit_num) => bit_num,
			None => return Err(Error::new_with_span("expression has no explicit size; use bit slices", expr.span.clone()))
		};
		
		rule.production_segments.push(expr);
	}
	
	if rule.production_bit_num % def.align_bits != 0
	{
		let full_span = begin_span.join(&parser.current().span);
		return Err(Error::new_with_span(format!("production is not aligned to `{}` bits", def.align_bits), full_span));
	}
	
	Ok(())
}


fn parse_variable_type(parser: &mut Parser) -> Result<VariableType, Error>
{
	let mut typ = VariableType
	{
		bit_num: 0,
		signed: false
	};
	
	let ident_token = try!(parser.expect_identifier()).clone();
	let ident = ident_token.identifier();
	
	match ident.chars().next().unwrap()
	{
		'u' => typ.signed = false,
		'i' => typ.signed = true,
		_ =>
			{ return Err(parser.make_error("invalid type", &ident_token.span)); }
	}
	
	match usize::from_str_radix(&ident[1..], 10)
	{
		Ok(bits) => typ.bit_num = bits,
		Err(..) =>
			{ return Err(parser.make_error("invalid type", &ident_token.span)); }
	}
	
	Ok(typ)
}