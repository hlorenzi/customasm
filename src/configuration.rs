use parser::{Parser, ParserError};
use tokenizer;
use numbits;


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
	pub fn from_src(src: &[char]) -> Result<Configuration, ParserError>
	{
		let mut config = Configuration
		{
			align_bits: 8,
			address_bits: 8,
			rules: Vec::new()
		};
		
		let tokens = tokenizer::tokenize(src);
		let mut parser = Parser::new(&tokens);
		try!(parse_directives(&mut config, &mut parser));
		try!(parse_rules(&mut config, &mut parser));
		
		Ok(config)
	}
}


fn parse_directives(config: &mut Configuration, parser: &mut Parser) -> Result<(), ParserError>
{
	while parser.match_operator(".")
	{
		let directive = try!(parser.expect_identifier()).clone();
		
		match directive.identifier().as_ref()
		{
			"align" => config.align_bits = try!(parser.expect_number()).number_usize(),
			"address" => config.address_bits = try!(parser.expect_number()).number_usize(),
			_ => return Err(ParserError::new(format!("unknown directive `{}`", directive.identifier()), directive.span))
		}
	}
	
	Ok(())
}


fn parse_rules(config: &mut Configuration, parser: &mut Parser) -> Result<(), ParserError>
{
	while !parser.is_over()
	{
		let pattern_segments = try!(parse_pattern(parser));
		try!(parser.expect_operator("->"));
		let production_segments = try!(parse_production(parser, &pattern_segments));
	
		config.rules.push(Rule
		{
			pattern_segments: pattern_segments,
			production_segments: production_segments
		});
		
		try!(parser.expect_operator(";"));
	}
	
	Ok(())
}


fn parse_pattern(parser: &mut Parser) -> Result<Vec<PatternSegment>, ParserError>
{
	let mut segments = Vec::new();
	
	while !parser.current().is_operator("->")
	{
		if parser.current().is_identifier()
		{
			let ident = try!(parser.expect_identifier());
			//println!("literal: {}", ident.identifier());
			segments.push(PatternSegment::Literal(ident.identifier().clone()));
		}
		else if parser.match_operator("{")
		{
			let name_token = try!(parser.expect_identifier()).clone();
			let name = name_token.identifier();
			
			if does_variable_exists(&segments, &name)
				{ return Err(ParserError::new(format!("duplicate variable `{}`", name), name_token.span)); }
			
			try!(parser.expect_operator(":"));
			
			let variable_type = try!(parse_variable_type(parser));
			
			//println!("variable: {}, signed: {}, bits: {}", name, variable_type.signed, variable_type.size_bits);
			segments.push(PatternSegment::Variable(name.clone(), variable_type));
			
			try!(parser.expect_operator("}"));
		}
		else if parser.current().is_any_operator()
		{
			let op = try!(parser.expect_any_operator());
			//println!("literal: {}", op.operator());
			segments.push(PatternSegment::Literal(op.operator().to_string()));
		}
		else
			{ return Err(ParserError::new("expected pattern".to_string(), parser.current().span)); }
	}
	
	Ok(segments)
}


fn parse_production(parser: &mut Parser, pattern: &Vec<PatternSegment>) -> Result<Vec<ProductionSegment>, ParserError>
{
	let mut segments = Vec::new();
	
	while !parser.current().is_operator(";")
	{
		if parser.current().is_number()
		{
			let size_token = try!(parser.expect_number()).clone();
			let size = size_token.number_usize();
			
			try!(parser.expect_operator("'"));
			let number_token = try!(parser.expect_number()).clone();
			let (radix, value_str) = number_token.number();
			
			let bits = match numbits::get_bits(size, radix, value_str)
			{
				Ok(bitvec) => bitvec,
				Err(msg) =>
					{ return Err(ParserError::new(msg, size_token.span)); }
			};
			
			//println!("produce bits: {:?}", bits);
			segments.push(ProductionSegment::Literal(bits));
		}
		else if parser.current().is_identifier()
		{
			let name_token = (*try!(parser.expect_identifier())).clone();
			let name = name_token.identifier();
			
			let variable_type = match get_variable_type(pattern, &name)
			{
				Some(typ) => typ,
				None => return Err(ParserError::new(format!("unknown variable `{}`", name), name_token.span))
			};
			
			let mut rightmost_bit = variable_type.size_bits;
			let mut leftmost_bit = 0;
			
			if parser.match_operator("[")
			{
				rightmost_bit = try!(parser.expect_number()).number_usize();
				try!(parser.expect_operator(":"));
				leftmost_bit = try!(parser.expect_number()).number_usize();
				try!(parser.expect_operator("]"));
			}
			
			//println!("produce variable: {}, bits: [{}:{}]", name, rightmost_bit, leftmost_bit);
			segments.push(ProductionSegment::Variable
			{
				name: name.clone(),
				leftmost_bit: leftmost_bit,
				rightmost_bit: rightmost_bit
			});
		}
		else
			{ return Err(ParserError::new("expected production".to_string(), parser.current().span)); }
	}
	
	Ok(segments)
}


fn parse_variable_type(parser: &mut Parser) -> Result<VariableType, ParserError>
{
	let mut typ = VariableType
	{
		size_bits: 0,
		signed: false
	};
	
	let ident_token = try!(parser.expect_identifier());
	let ident = ident_token.identifier();
	
	match ident.chars().next().unwrap()
	{
		'u' => typ.signed = false,
		'i' => typ.signed = true,
		_ =>
			{ return Err(ParserError::new("invalid type".to_string(), ident_token.span)); }
	}
	
	match usize::from_str_radix(&ident[1..], 10)
	{
		Ok(bits) => typ.size_bits = bits,
		Err(..) =>
			{ return Err(ParserError::new("invalid type".to_string(), ident_token.span)); }
	}
	
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