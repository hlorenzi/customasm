use util::error::Error;
use util::tokenizer::{Token, TokenKind, CharIndex, Span};
use std::rc::Rc;


pub struct Parser<'tok>
{
	tokens: &'tok [Token],
	index: usize,
	end_token: Token
}


impl<'tok> Parser<'tok>
{
	pub fn new_from_index(tokens: &'tok [Token], start_index: usize) -> Parser<'tok>
	{
		let end_index =
			if tokens.len() > 0
				{ tokens[tokens.len() - 1].span.end }
			else
				{ CharIndex::new() };
			
		let end_filename =
			if tokens.len() > 0
				{ tokens[0].span.file.as_ref() }
			else
				{ "<unknown>" };
			
		let end_token = Token
		{
			span: Span::new(Rc::new(end_filename.to_string()), end_index, end_index),
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
	
	
	pub fn clone_from_current(&self) -> Parser<'tok>
	{
		Parser::new_from_index(self.tokens, self.index)
	}
	
	
	pub fn get_filename(&self) -> &str
	{
		if self.tokens.len() > 0
			{ self.tokens[0].span.file.as_ref() }
		else
			{ panic!("no token in parser") }
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
			{ Err(Error::new_with_span("expected line break", self.current().span.clone())) }
	}
	
	
	pub fn expect_linebreak_or_end(&mut self) -> Result<&Token, Error>
	{
		if self.current().is_linebreak() || self.index >= self.tokens.len()
			{ Ok(self.advance()) }
		else
			{ Err(Error::new_with_span("expected line break", self.current().span.clone())) }
	}
	
	
	pub fn expect_identifier(&mut self) -> Result<(String, Span), Error>
	{
		if self.current().is_identifier()
		{
			let token = self.advance();
			let ident = token.identifier().clone();
			let span = token.span.clone();
			Ok((ident, span))
		}
		else
			{ Err(Error::new_with_span("expected identifier", self.current().span.clone())) }
	}
	
	
	pub fn expect_number(&mut self) -> Result<(usize, Span), Error>
	{
		if self.current().is_number()
		{
			let token = self.advance();
			let value = token.number_value();
			let span = token.span.clone();
			Ok((value, span))
		}
		else
			{ Err(Error::new_with_span("expected number", self.current().span.clone())) }
	}
	
	
	pub fn expect_number_str(&mut self) -> Result<(usize, &String, Span), Error>
	{
		if self.current().is_number()
		{
			let token = self.advance();
			let (radix, value_str) = token.number();
			let span = token.span.clone();
			Ok((radix, value_str, span))
		}
		else
			{ Err(Error::new_with_span("expected number", self.current().span.clone())) }
	}
	
	
	pub fn expect_string(&mut self) -> Result<(String, Span), Error>
	{
		if self.current().is_string()
		{
			let token = self.advance();
			let string = token.string().clone();
			let span = token.span.clone();
			Ok((string, span))
		}
		else
			{ Err(Error::new_with_span("expected string", self.current().span.clone())) }
	}
	
	
	pub fn expect_any_operator(&mut self) -> Result<(&'static str, Span), Error>
	{
		if self.current().is_any_operator()
		{
			let token = self.advance();
			let op = token.operator();
			let span = token.span.clone();
			Ok((op, span))
		}
		else
			{ Err(Error::new_with_span("expected operator", self.current().span.clone())) }
	}
	
	
	pub fn expect_operator(&mut self, op: &str) -> Result<(), Error>
	{
		if self.current().is_operator(op)
		{
			self.advance();
			Ok(())
		}
		else
			{ Err(Error::new_with_span(format!("expected `{}`", op), self.current().span.clone())) }
	}
}