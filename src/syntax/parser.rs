use diagn::Message;
use syntax::{Token, TokenKind};


pub struct Parser<'t>
{
	tokens: &'t [Token],
	index: usize,
	index_prev: usize,
	read_linebreak: bool
}


impl<'t> Parser<'t>
{
	pub fn new(tokens: &'t [Token]) -> Parser<'t>
	{
		let mut parser = Parser
		{
			tokens: tokens,
			index: 0,
			index_prev: 0,
			read_linebreak: false
		};
		
		parser.skip_ignorable();
		parser
	}
	
	
	fn skip_ignorable(&mut self)
	{
		while self.index < self.tokens.len() - 1 &&
			self.tokens[self.index].kind.ignorable()
		{
			if self.tokens[self.index].kind == TokenKind::LineBreak
				{ self.read_linebreak = true; }
				
			self.index += 1;
		}
	}
	
	
	pub fn advance(&mut self) -> Token
	{
		self.index_prev = self.index;
	
		let token = self.tokens[self.index].clone();
		
		if self.index < self.tokens.len() - 1
			{ self.index += 1; }
		
		self.read_linebreak = false;
		self.skip_ignorable();
		token
	}
	
	
	pub fn next_is(&self, nth: usize, kind: TokenKind) -> bool
	{
		if self.index + nth >= self.tokens.len()
			{ false }
		else
			{ self.tokens[self.index + nth].kind == kind }
	}
	
	
	pub fn maybe_expect(&mut self, kind: TokenKind) -> Option<Token>
	{
		if self.next_is(0, kind)
			{ Some(self.advance()) }
		else
			{ None }
	}
	
	
	pub fn expect(&mut self, kind: TokenKind) -> Result<Token, Message>
	{
		match self.maybe_expect(kind)
		{
			Some(token) => Ok(token),
			None =>
			{
				let descr = format!("expected {}", kind.printable());
				Err(Message::error_span(descr, &self.tokens[self.index_prev].span.after()))
			}
		}
	}
	
	
	pub fn expect_msg<S>(&mut self, kind: TokenKind, descr: S) -> Result<Token, Message>
	where S: Into<String>
	{
		match self.maybe_expect(kind)
		{
			Some(token) => Ok(token),
			None => Err(Message::error_span(descr, &self.tokens[self.index_prev].span.after()))
		}
	}
	
	
	pub fn expect_linebreak(&mut self) -> Result<(), Message>
	{
		if self.read_linebreak
			{ Ok(()) }
		else
			{ Err(Message::error_span("expected line break", &self.tokens[self.index_prev].span.after())) }
	}
}