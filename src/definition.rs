use util::tokenizer;
use util::parser::{Parser, ParserError};
use util::bitvec::BitVec;
use rule::{Rule, PatternSegment, ProductionSegment, VariableType};


pub struct Definition
{
	pub align_bits: usize,
	pub address_bits: usize,
	pub rules: Vec<Rule>
}


pub fn parse(src: &[char]) -> Result<Definition, ParserError>
{
	let mut def = Definition
	{
		align_bits: 8,
		address_bits: 8,
		rules: Vec::new()
	};
	
	let tokens = tokenizer::tokenize(src);
	let mut parser = Parser::new(&tokens);
	try!(parse_directives(&mut def, &mut parser));
	try!(parse_rules(&mut def, &mut parser));
	
	Ok(def)
}


fn parse_directives(def: &mut Definition, parser: &mut Parser) -> Result<(), ParserError>
{
	while parser.match_operator(".")
	{
		let directive = try!(parser.expect_identifier()).clone();
		
		match directive.identifier().as_ref()
		{
			"align" => def.align_bits = try!(parser.expect_number()).number_usize(),
			"address" => def.address_bits = try!(parser.expect_number()).number_usize(),
			_ => return Err(ParserError::new(format!("unknown directive `{}`", directive.identifier()), directive.span))
		}
		
		try!(parser.expect_separator_linebreak());
	}
	
	Ok(())
}


fn parse_rules(def: &mut Definition, parser: &mut Parser) -> Result<(), ParserError>
{
	while !parser.is_over()
	{
		let mut rule = Rule::new();
	
		let rule_span = parser.current().span;
		
		try!(parse_pattern(parser, &mut rule));
		try!(parser.expect_operator("->"));
		try!(parse_production(parser, &mut rule));
		
		if rule.production_bit_num % def.align_bits != 0
			{ return Err(ParserError::new(format!("production is not aligned to `{}` bits", def.align_bits), rule_span)); }
	
		def.rules.push(rule);
		
		try!(parser.expect_separator_linebreak());
	}
	
	Ok(())
}


fn parse_pattern(parser: &mut Parser, rule: &mut Rule) -> Result<(), ParserError>
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
				{ return Err(ParserError::new(format!("duplicate argument `{}`", name), name_token.span)); }
			
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
			{ return Err(ParserError::new("expected pattern".to_string(), parser.current().span)); }
	}
	
	Ok(())
}


fn parse_production(parser: &mut Parser, rule: &mut Rule) -> Result<(), ParserError>
{
	while !parser.current().is_linebreak_or_end()
	{
		if parser.current().is_number()
		{
			let size_token = try!(parser.expect_number()).clone();
			let size = size_token.number_usize();
			
			try!(parser.expect_operator("'"));
			let number_token = try!(parser.expect_number()).clone();
			let (radix, value_str) = number_token.number();
			
			let bitvec = match BitVec::new_from_str_sized(size, radix, value_str)
			{
				Ok(bitvec) => bitvec,
				Err(msg) => return Err(ParserError::new(msg, size_token.span))
			};
			
			rule.production_bit_num += bitvec.len();
			rule.production_segments.push(ProductionSegment::Literal(bitvec));
		}
		else if parser.current().is_identifier()
		{
			let name_token = try!(parser.expect_identifier()).clone();
			let name = name_token.identifier();
			
			let arg_index = match rule.get_argument(&name)
			{
				Some(arg_index) => arg_index,
				None => return Err(ParserError::new(format!("unknown argument `{}`", name), name_token.span))
			};
			
			let typ = rule.get_argument_type(arg_index);
			
			let mut rightmost_bit = typ.bit_num;
			let mut leftmost_bit = 0;
			
			if parser.match_operator("[")
			{
				rightmost_bit = try!(parser.expect_number()).number_usize();
				try!(parser.expect_operator(":"));
				leftmost_bit = try!(parser.expect_number()).number_usize();
				try!(parser.expect_operator("]"));
			}
			
			rule.production_bit_num += rightmost_bit - leftmost_bit;
			rule.production_segments.push(ProductionSegment::Argument
			{
				index: arg_index,
				leftmost_bit: leftmost_bit,
				rightmost_bit: rightmost_bit
			});
		}
		else
			{ return Err(ParserError::new("expected production".to_string(), parser.current().span)); }
	}
	
	Ok(())
}


fn parse_variable_type(parser: &mut Parser) -> Result<VariableType, ParserError>
{
	let mut typ = VariableType
	{
		bit_num: 0,
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
		Ok(bits) => typ.bit_num = bits,
		Err(..) =>
			{ return Err(ParserError::new("invalid type".to_string(), ident_token.span)); }
	}
	
	Ok(typ)
}