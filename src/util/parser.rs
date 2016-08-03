use util::error::Error;
use util::tokenizer::{Token, TokenKind, CharIndex, Span};
use std::rc::Rc;


pub struct Parser<'f, 'tok>
{
	filename: &'f str,
	tokens: &'tok [Token],
	index: usize,
	end_token: Token
}


impl<'f, 'tok> Parser<'f, 'tok>
{
	pub fn new_from_index(filename: &'f str, tokens: &'tok [Token], start_index: usize) -> Parser<'f, 'tok>
	{
		let end_index =
			if tokens.len() > 0
				{ tokens[tokens.len() - 1].span.end }
			else
				{ CharIndex::new() };
				
		let end_token = Token
		{
			span: Span::new(Rc::new(filename.to_string()), end_index, end_index),
			kind: TokenKind::End
		};
		
		Parser
		{
			filename: filename,
			tokens: tokens,
			index: start_index,
			end_token: end_token
		}
	}
	
	
	pub fn new(filename: &'f str, tokens: &'tok [Token]) -> Parser<'f, 'tok>
	{
		Parser::new_from_index(filename, tokens, 0)
	}
	
	
	pub fn clone_from_current(&self) -> Parser<'f, 'tok>
	{
		Parser::new_from_index(self.filename, self.tokens, self.index)
	}
	
	
	pub fn get_filename(&self) -> &'f str
	{
		self.filename
	}
	
	
	pub fn make_error<S>(&self, msg: S, span: &Span) -> Error
	where S: Into<String>
	{
		Error::new_with_span(msg.into(), span.clone())
	}
	
	
	pub fn current(&self) -> &Token
	{
		self.next(0)
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
	
	
	pub fn expect_linebreak(&mut self) -> Result<&Token, Error>
	{
		if self.current().is_linebreak()
			{ Ok(self.advance()) }
		else
			{ Err(self.make_error("expected line break", &self.current().span)) }
	}
	
	
	pub fn expect_separator_linebreak(&mut self) -> Result<&Token, Error>
	{
		if self.current().is_linebreak() || self.index >= self.tokens.len()
			{ Ok(self.advance()) }
		else
			{ Err(self.make_error("expected line break", &self.current().span)) }
	}
	
	
	pub fn expect_identifier(&mut self) -> Result<&Token, Error>
	{
		if self.current().is_identifier()
			{ Ok(self.advance()) }
		else
			{ Err(self.make_error("expected identifier", &self.current().span)) }
	}
	
	
	pub fn expect_number(&mut self) -> Result<&Token, Error>
	{
		if self.current().is_number()
			{ Ok(self.advance()) }
		else
			{ Err(self.make_error("expected number", &self.current().span)) }
	}
	
	
	pub fn expect_string(&mut self) -> Result<&Token, Error>
	{
		if self.current().is_string()
			{ Ok(self.advance()) }
		else
			{ Err(self.make_error("expected string", &self.current().span)) }
	}
	
	
	pub fn expect_any_operator(&mut self) -> Result<&Token, Error>
	{
		if self.current().is_any_operator()
			{ Ok(self.advance()) }
		else
			{ Err(self.make_error("expected operator", &self.current().span)) }
	}
	
	
	pub fn expect_operator(&mut self, op: &str) -> Result<(), Error>
	{
		if self.current().is_operator(op)
		{
			self.advance();
			Ok(())
		}
		else
			{ Err(self.make_error(format!("expected `{}`", op), &self.current().span)) }
	}
}