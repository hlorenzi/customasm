use diagn::Message;
use syntax::{TokenKind, tokenize, Parser};
use asm::AssemblerState;


pub struct AssemblerParser<'a, 'b>
where 'b: 'a
{
	pub state: &'a mut AssemblerState<'b>,
	pub parser: Parser<'a>
}
	
	
impl<'a, 'b> AssemblerParser<'a, 'b>
{
	pub fn parse_file<S>(state: &mut AssemblerState, filename: S) -> Result<(), Message>
	where S: Into<String>
	{
		let filename_owned = filename.into();
		let chars = state.fileserver.get_chars(&filename_owned)?;
		let tokens = tokenize(state.reporter, filename_owned, &chars);
		
		let mut parser = AssemblerParser
		{
			state: state,
			parser: Parser::new(&tokens)
		};
		
		parser.parse()
	}
	
	
	fn parse(&mut self) -> Result<(), Message>
	{
		while !self.parser.is_over()
		{
			self.parse_line()?;
			self.parser.expect_linebreak()?;
		}
		
		Ok(())
	}
	
	
	fn parse_line(&mut self) -> Result<(), Message>
	{
		self.parse_instruction()
	}
	
	
	fn parse_instruction(&mut self) -> Result<(), Message>
	{
		let instr_span_start = self.parser.next().span;
		
		let (instr_match, new_parser) = match self.state.pattern_matcher.parse_match(self.parser.clone())
		{
			Some((instr_match, new_parser)) => (instr_match, new_parser),
			None =>
			{
				self.parser.skip_until_linebreak();
				let instr_span = instr_span_start.join(&self.parser.prev().span);
				
				self.state.reporter.message(Message::error_span("no match for instruction found", &instr_span));
				return Ok(());
			}
		};
		
		self.parser = new_parser;
		
		Ok(())
	}
}