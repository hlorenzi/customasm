use parser::{Parser, ParserError};


pub struct Configuration
{
	align_bits: usize,
	address_bits: usize,
	rules: Vec<Rule>
}


struct Rule
{
	pattern_segments: Vec<PatternSegment>,
	production_segments: Vec<ProductionSegment>
}


enum PatternSegment
{
	Literal(String),
	Variable(String, VariableType)
}


struct VariableType
{
	size_bits: usize,
	signed: bool
}


enum ProductionSegment
{
	Literal(Vec<bool>),
	Variable(ProductionVariable)
}


struct ProductionVariable
{
	variable: String,
	start_bit: Option<usize>,
	end_bit: Option<usize>
}


impl Configuration
{
	pub fn parse(src: &mut Iterator<Item = char>) -> Result<Configuration, ParserError>
	{
		let mut config = Configuration
		{
			align_bits: 8,
			address_bits: 8,
			rules: Vec::new()
		};
		
		let mut parser = Parser::new(src);
		try!(parse_directives(&mut config, &mut parser));
		try!(parse_rules(&mut config, &mut parser));
		
		Ok(config)
	}
}


fn parse_directives(config: &mut Configuration, parser: &mut Parser) -> Result<(), ParserError>
{
	parser.skip_white();
	while parser.matches('!')
	{
		parser.skip_white();
		let directive = try!(parser.get_identifier());
		parser.skip_white();
		try!(parser.expect(':'));
		parser.skip_white();
		
		match directive.as_ref()
		{
			"align" =>
			{
				let value = try!(parser.get_integer());
				config.align_bits = value.parse::<usize>().unwrap();
			}
			"address" =>
			{
				let value = try!(parser.get_integer());
				config.address_bits = value.parse::<usize>().unwrap();
			}
			_ => return Err(parser.error(format!("unknown directive `{}`", directive)))
		}
		
		parser.skip_white();
		try!(parser.expect(';'));
		parser.skip_white();
	}
	
	Ok(())
}


fn parse_rules(config: &mut Configuration, parser: &mut Parser) -> Result<(), ParserError>
{
	parser.skip_white();
	while !parser.is_over()
	{
		let pattern_segments = try!(parse_pattern(parser));
		try!(parser.expect_str("->"));
		let production_segments = try!(parse_production(parser));
	
		config.rules.push(Rule
		{
			pattern_segments: pattern_segments,
			production_segments: production_segments
		});
		
		parser.skip_white();
		try!(parser.expect(';'));
		parser.skip_white();
	}
	
	Ok(())
}


fn parse_pattern(parser: &mut Parser) -> Result<Vec<PatternSegment>, ParserError>
{
	let mut segments = Vec::new();
	
	parser.skip_white();
	while !parser.current_is_str("->")
	{
		parser.skip_white();
		
		if parser.current_is_pattern()
		{
			let literal = try!(parser.get_pattern());
			println!("literal: {}", literal);
			segments.push(PatternSegment::Literal(literal));
			parser.skip_white();
		}
		else if parser.matches('{')
		{
			parser.skip_white();
			let variable_name = try!(parser.get_identifier());
			parser.skip_white();
			try!(parser.expect(':'));
			parser.skip_white();
			let variable_type = try!(parse_variable_type(parser));
			parser.skip_white();
			
			println!("variable: {}, signed: {}, bits: {}", variable_name, variable_type.signed, variable_type.size_bits);
			segments.push(PatternSegment::Variable(variable_name, variable_type));
			
			try!(parser.expect('}'));
			parser.skip_white();
		}
		else
			{ return Err(parser.error("expected `->`".to_string())); }
	}
	
	parser.skip_white();
	Ok(segments)
}


fn parse_production(parser: &mut Parser) -> Result<Vec<ProductionSegment>, ParserError>
{
	let mut segments = Vec::new();
	
	parser.skip_white();
	while !parser.current_is(';')
	{
		parser.skip_white();
		
		if parser.current_is_number()
		{
			let bits = try!(parser.get_bits());
			println!("produce bits: {:?}", bits);
			segments.push(ProductionSegment::Literal(bits));
			parser.skip_white();
		}
		else
			{ return Err(parser.error("expected `;`".to_string())); }
	}
	
	parser.skip_white();
	Ok(segments)
}


fn parse_variable_type(parser: &mut Parser) -> Result<VariableType, ParserError>
{
	let mut typ = VariableType
	{
		size_bits: 0,
		signed: false
	};
	
	if parser.matches('u')
		{ typ.signed = false; }
	else if parser.matches('i')
		{ typ.signed = true; }
	else
		{ return Err(parser.error("invalid type".to_string())); }
	
	typ.size_bits = try!(parser.get_integer()).parse::<usize>().unwrap();
	Ok(typ)
}


















