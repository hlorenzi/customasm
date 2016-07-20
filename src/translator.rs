use parser::{Parser, ParserError};
use configuration::Configuration;


struct Translator<'cfg>
{
	config: &'cfg Configuration,
	cur_address: usize,
	cur_output: usize,
}


pub fn translate(config: &Configuration, src: &mut Iterator<Item = char>) -> Result<Vec<bool>, ParserError>
{
	let mut translator = Translator
	{
		config: config,
		cur_address: 0,
		cur_output: 0
	};
	
	let mut output = Vec::new();
	
	let mut parser = Parser::new(src);
	parser.skip_white();
	
	while !parser.is_over()
	{
		parser.skip_white();
		
		if parser.current_is('.')
			{ try!(translate_directive(&mut translator, &mut parser)); }
		else
			{ try!(translate_instruction(&mut translator, &mut parser, &mut output)); }
			
		parser.skip_white();
	}
	
	Ok(output)
}


fn translate_directive(translator: &mut Translator, parser: &mut Parser) -> Result<(), ParserError>
{
	try!(parser.expect('.'));
	let directive = try!(parser.get_identifier());
	parser.skip_white();
	
	match directive.as_ref()
	{
		"address" => translator.cur_address = try!(parser.get_usize()),
		"output" => translator.cur_output = try!(parser.get_usize()),
		_ => return Err(parser.error(format!("unknown directive `{}`", directive)))
	}
	
	parser.skip_white();
	Ok(())
}


fn translate_instruction(translator: &mut Translator, parser: &mut Parser, output: &mut Vec<bool>) -> Result<(), ParserError>
{
	let line_str = parser.get_line();
	
	for rule in translator.config.rules.iter()
	{
		
	}
	
	Ok(())
}