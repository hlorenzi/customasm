use util::error::Error;
use util::expression::Expression;
use util::parser::Parser;
use util::tokenizer;
use rule::{Rule, PatternSegment};


pub struct Definition
{
	pub align_bits: usize,
	pub rules: Vec<Rule>
}


pub fn parse(src_filename: &str, src: &[char]) -> Result<Definition, Error>
{
	let mut def = Definition
	{
		align_bits: 8,
		rules: Vec::new()
	};
	
	let tokens = tokenizer::tokenize(src_filename, src);
	let mut parser = Parser::new(&tokens);
	try!(parse_directives(&mut def, &mut parser));
	try!(parse_rules(&mut def, &mut parser));
	
	Ok(def)
}


fn parse_directives(def: &mut Definition, parser: &mut Parser) -> Result<(), Error>
{
	while parser.match_operator(".")
	{
		let (directive, directive_span) = try!(parser.expect_identifier());
		
		match directive.as_ref()
		{
			"align" => def.align_bits = try!(parser.expect_number()).0,
			
			_ => return Err(Error::new_with_span(format!("unknown directive `{}`", directive), directive_span))
		}
		
		try!(parser.expect_linebreak_or_end());
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
		
		try!(parser.expect_linebreak_or_end());
	}
	
	Ok(())
}


fn parse_pattern(parser: &mut Parser, rule: &mut Rule) -> Result<(), Error>
{
	loop
	{
		if parser.current().is_identifier()
		{
			let (ident, _) = try!(parser.expect_identifier());
			rule.pattern_segments.push(PatternSegment::Exact(ident));
		}
		
		else if parser.match_operator("{")
		{
			let (name, name_span) = try!(parser.expect_identifier());
			
			if name == "pc"
				{ return Err(Error::new_with_span("reserved parameter name `pc`", name_span)); }
				
			if rule.check_parameter_exists(&name)
				{ return Err(Error::new_with_span(format!("duplicate parameter `{}`", name), name_span)); }
				
			let allow_unresolved = !parser.match_operator("!");
			
			let constraint =
				if parser.match_operator(":")
					{ Some(try!(Expression::new_by_parsing_checked(parser, &|name| name == "_" || name == "pc"))) }
				else
					{ None };
			
			let param_index = rule.add_parameter(name, allow_unresolved, constraint);
			rule.pattern_segments.push(PatternSegment::Parameter(param_index));
			
			try!(parser.expect_operator("}"));
		}
		
		else if parser.current().is_any_operator()
		{
			let (op, _) = try!(parser.expect_any_operator());
			rule.pattern_segments.push(PatternSegment::Exact(op.to_string()));
		}
		
		else
			{ return Err(Error::new_with_span("expected pattern", parser.current().span.clone())); }
		
		
		if parser.current().is_operator("->")
			{ break; }
	}
	
	Ok(())
}


fn parse_production(def: &mut Definition, parser: &mut Parser, rule: &mut Rule) -> Result<(), Error>
{
	let begin_span = parser.current().span.clone();
	
	loop
	{
		let expr = try!(Expression::new_by_parsing_checked(parser,
			&|name| name == "pc" || rule.check_parameter_exists(name)));
		
		rule.production_bit_num += match expr.get_explicit_bit_num()
		{
			Some(bit_num) => bit_num,
			None => return Err(Error::new_with_span("expression has no explicit size; use bit slices", expr.span.clone()))
		};
		
		rule.production_segments.push(expr);
		
		if parser.current().is_linebreak_or_end()
			{ break; }
	}
	
	if rule.production_bit_num % def.align_bits != 0
	{
		let full_span = begin_span.join(&parser.current().span);
		return Err(Error::new_with_span(format!("production is not aligned to `{}` bits", def.align_bits), full_span));
	}
	
	Ok(())
}