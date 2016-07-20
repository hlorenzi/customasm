use parser::{Parser, ParserError};


pub struct Configuration
{
	pub align_bits: usize,
	pub address_bits: usize,
	pub rules: Vec<Rule>
}


pub struct Rule
{
	pub pattern_segments: Vec<PatternSegment>,
	pub production_segments: Vec<ProductionSegment>
}


pub enum PatternSegment
{
	Literal(String),
	Variable(String, VariableType)
}


#[derive(Copy, Clone)]
pub struct VariableType
{
	pub size_bits: usize,
	pub signed: bool
}


pub enum ProductionSegment
{
	Literal(Vec<bool>),
	Variable
	{
		name: String,
		leftmost_bit: usize,
		rightmost_bit: usize
	}
}


impl Configuration
{
	pub fn from_src(src: &mut Iterator<Item = char>) -> Result<Configuration, ParserError>
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
	while parser.matches('.')
	{
		parser.skip_white();
		let directive = try!(parser.get_identifier());
		parser.skip_white();
		
		match directive.as_ref()
		{
			"align" => config.align_bits = try!(parser.get_usize()),
			"address" => config.address_bits = try!(parser.get_usize()),
			_ => return Err(parser.error(format!("unknown directive `{}`", directive)))
		}
		
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
		let production_segments = try!(parse_production(parser, &pattern_segments));
	
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
			if does_variable_exists(&segments, &variable_name)
				{ return Err(parser.error(format!("duplicate variable `{}`", variable_name))); }
			
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


fn parse_production(parser: &mut Parser, pattern: &Vec<PatternSegment>) -> Result<Vec<ProductionSegment>, ParserError>
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
		else if parser.current_is_identifier()
		{
			let variable_name = try!(parser.get_identifier());
			let variable_type = match get_variable_type(pattern, &variable_name)
			{
				Some(typ) => typ,
				None => return Err(parser.error(format!("unknown variable `{}`", variable_name)))
			};
			
			let mut rightmost_bit = variable_type.size_bits;
			let mut leftmost_bit = 0;
			
			parser.skip_white();
			if parser.matches('[')
			{
				parser.skip_white();
				rightmost_bit = try!(parser.get_usize());
				parser.skip_white();
				try!(parser.expect(':'));
				parser.skip_white();
				leftmost_bit = try!(parser.get_usize());
				parser.skip_white();
				try!(parser.expect(']'));
				parser.skip_white();
			}
			
			println!("produce variable: {}, bits: [{}:{}]", variable_name, rightmost_bit, leftmost_bit);
			segments.push(ProductionSegment::Variable
			{
				name: variable_name,
				leftmost_bit: leftmost_bit,
				rightmost_bit: rightmost_bit
			});
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
	
	typ.size_bits = try!(parser.get_usize());
	Ok(typ)
}


fn does_variable_exists(pattern_segments: &Vec<PatternSegment>, name: &str) -> bool
{
	for segment in pattern_segments
	{
		match segment
		{
			&PatternSegment::Variable(ref n, _) =>
			{
				if n == name
					{ return true; }
			}
			_ => { }
		}
	}
	
	false
}


fn get_variable_type(pattern_segments: &Vec<PatternSegment>, name: &str) -> Option<VariableType>
{
	for segment in pattern_segments
	{
		match segment
		{
			&PatternSegment::Variable(ref n, ref typ) =>
			{
				if n == name
					{ return Some(*typ); }
			}
			_ => { }
		}
	}
	
	None
}


















