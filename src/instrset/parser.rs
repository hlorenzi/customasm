use diagn::{Message, Reporter};
use syntax::{Token, TokenKind, Parser};
use syntax::excerpt_as_usize;
use ::InstrSet;


pub struct InstrSetParser<'a, 't>
{
	instrset: &'a mut InstrSet,
	
	reporter: &'a mut Reporter,
	parser: Parser<'t>,
	
	align_was_set: bool
}


impl<'a, 't> InstrSetParser<'a, 't>
{
	pub fn new(reporter: &'a mut Reporter, tokens: &'t [Token], instrset: &'a mut InstrSet) -> InstrSetParser<'a, 't>
	{
		InstrSetParser
		{
			instrset: instrset,
			
			reporter: reporter,
			parser: Parser::new(tokens),
			
			align_was_set: false
		}
	}
	

	pub fn parse(&mut self) -> Result<(), Message>
	{
		self.parse_directives()?;
		self.parse_rules()
	}
	

	fn parse_directives(&mut self) -> Result<(), Message>
	{
		while self.parser.maybe_expect(TokenKind::Hash).is_some()
		{
			let tk_name = self.parser.expect_msg(TokenKind::Identifier, "expected directive name")?;
			match tk_name.excerpt.as_ref().unwrap().as_ref()
			{
				"align" => self.parse_directive_align(&tk_name)?,
				
				_ => return Err(Message::error_span("unknown directive", &tk_name.span))
			}
		}
	
		Ok(())
	}
	
	
	fn parse_directive_align(&mut self, tk_name: &Token) -> Result<(), Message>
	{
		let tk_align = self.parser.expect_msg(TokenKind::Number, "expected alignment value")?;
		self.parser.expect_linebreak()?;
		
		if self.align_was_set
			{ return Err(Message::error_span("duplicate align directive", &tk_name.span)); }
			
		self.instrset.align = excerpt_as_usize(&tk_align.excerpt.unwrap(), &tk_align.span)?;
		self.align_was_set = true;
		
		Ok(())
	}
	

	fn parse_rules(&mut self) -> Result<(), Message>
	{
		Ok(())
	}
}