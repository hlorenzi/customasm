use tokenizer::{Token, TokenKind, Span};


pub struct Parser<'tok>
{
	tokens: &'tok [Token],
	index: usize,
	end_token: Token
}


#[derive(Debug)]
pub struct ParserError
{
	pub msg: String,
	pub span: Span
}


impl ParserError
{
	pub fn new(msg: String, span: Span) -> ParserError
	{
		ParserError
		{
			msg: msg,
			span: span
		}
	}
}


impl<'tok> Parser<'tok>
{
	pub fn new_from_index(tokens: &'tok [Token], start_index: usize) -> Parser<'tok>
	{
		let end_index =
			if tokens.len() > 0
				{ tokens[tokens.len() - 1].span.end }
			else
				{ 0 };
				
		let end_token = Token
		{
			span: Span { start: end_index, end: end_index },
			kind: TokenKind::End
		};
		
		Parser
		{
			tokens: tokens,
			index: start_index,
			end_token: end_token
		}
	}
	
	
	pub fn new(tokens: &'tok [Token]) -> Parser<'tok>
	{
		Parser::new_from_index(tokens, 0)
	}
	
	
	pub fn clone_from_current(&self) -> Parser
	{
		Parser::new_from_index(&self.tokens[self.index..], 0)
	}
	
	
	pub fn current(&self) -> &Token
	{
		self.next(0)
	}
	
	
	pub fn current_index(&self) -> usize
	{
		self.index
	}
	
	
	pub fn next(&self, index: usize) -> &Token
	{
		if self.index + index >= self.tokens.len()
			{ &self.end_token }
		else
			{ &self.tokens[self.index + index] }
	}
	
	
	pub fn is_over(&self) -> bool
	{
		self.index >= self.tokens.len()
	}
	
	
	pub fn advance(&mut self) -> &Token
	{
		self.index += 1;
		if self.index - 1 >= self.tokens.len()
			{ &self.end_token }
		else
			{ &self.tokens[self.index - 1] }
	}
	
	
	pub fn match_operator(&mut self, op: &str) -> bool
	{
		if self.current().is_operator(op)
		{
			self.advance();
			true
		}
		else
			{ false }
	}
	
	
	pub fn expect_identifier(&mut self) -> Result<&Token, ParserError>
	{
		if self.current().is_identifier()
			{ Ok(self.advance()) }
		else
			{ Err(ParserError::new("expected identifier".to_string(), self.current().span)) }
	}
	
	
	pub fn expect_number(&mut self) -> Result<&Token, ParserError>
	{
		if self.current().is_number()
			{ Ok(self.advance()) }
		else
			{ Err(ParserError::new("expected number".to_string(), self.current().span)) }
	}
	
	
	pub fn expect_any_operator(&mut self) -> Result<&Token, ParserError>
	{
		if self.current().is_any_operator()
			{ Ok(self.advance()) }
		else
			{ Err(ParserError::new("expected operator".to_string(), self.current().span)) }
	}
	
	
	pub fn expect_operator(&mut self, op: &str) -> Result<(), ParserError>
	{
		if self.current().is_operator(op)
		{
			self.advance();
			Ok(())
		}
		else
			{ Err(ParserError::new(format!("expected `{}`", op), self.current().span)) }
	}
}