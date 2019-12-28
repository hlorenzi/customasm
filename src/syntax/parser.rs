use crate::diagn::RcReport;
use crate::syntax::{Token, TokenKind, excerpt_as_usize};


pub struct Parser
{
	pub report: RcReport,
	tokens: Vec<Token>,
	index: usize,
	index_prev: usize,
	read_linebreak: bool,
	partial_index: usize
}


pub struct ParserState
{
	index: usize,
	index_prev: usize,
	read_linebreak: bool,
	partial_index: usize
}


impl Parser
{
	pub fn new(report: RcReport, tokens: Vec<Token>) -> Parser
	{
		assert!(tokens[tokens.len() - 1].kind == TokenKind::End);
	
		let mut parser = Parser
		{
			report: report,
			tokens: tokens,
			index: 0,
			index_prev: 0,
			read_linebreak: false,
			partial_index: 0
		};
		
		parser.skip_ignorable();
		parser
	}
	
	
	pub fn save(&self) -> ParserState
	{
		ParserState
		{
			index: self.index,
			index_prev: self.index_prev,
			read_linebreak: self.read_linebreak,
			partial_index: self.partial_index
		}
	}
	
	
	pub fn restore(&mut self, state: ParserState)
	{
		self.index = state.index;
		self.index_prev = state.index_prev;
		self.read_linebreak = state.read_linebreak;
		self.partial_index = state.partial_index;
	}
	
	
	pub fn is_over(&self) -> bool
	{
		self.index >= self.tokens.len() - 1
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
		if self.is_at_partial()
			{ panic!("at partial"); }

		self.index_prev = self.index;
	
		let token = self.tokens[self.index].clone();
		
		if self.index < self.tokens.len() - 1
			{ self.index += 1; }
		
		self.read_linebreak = false;
		self.skip_ignorable();
		token
	}


	pub fn advance_partial(&mut self) -> char
	{
		if self.tokens[self.index].kind == TokenKind::End
			{ return '\0'; }

		let sliced = unsafe { self.tokens[self.index].text().get_unchecked(self.partial_index..) };
		let mut char_indices = sliced.char_indices();
		let c = char_indices.next().unwrap().1;

		if let Some((index, _)) = char_indices.next()
		{
			self.partial_index += index;
		}
		else
		{
			self.partial_index = 0;
			self.advance();
		}

		c
	}
	
	
	pub fn skip_until_linebreak(&mut self)
	{
		while !self.is_over() && !self.next_is_linebreak()
			{ self.advance(); }
	}
	
	
	pub fn next(&self) -> Token
	{
		self.tokens[self.index].clone()
	}
	
	
	pub fn prev(&self) -> Token
	{
		self.tokens[self.index_prev].clone()
	}
	
	
	pub fn clear_linebreak(&mut self)
	{
		self.read_linebreak = false;
	}


	pub fn is_at_partial(&self) -> bool
	{
		self.partial_index != 0
	}
	
	
	pub fn next_is(&self, mut nth: usize, kind: TokenKind) -> bool
	{
		let mut index = self.index;
		
		while nth > 0 && index < self.tokens.len() - 1
		{
			nth -= 1;
			index += 1;
			while self.tokens[index].kind.ignorable() && index < self.tokens.len() - 1
				{ index += 1; }
		}
		
		self.tokens[index].kind == kind
	}
	
	
	pub fn maybe_expect(&mut self, kind: TokenKind) -> Option<Token>
	{
		if self.next_is(0, kind)
			{ Some(self.advance()) }
		else
			{ None }
	}
	
	
	pub fn expect(&mut self, kind: TokenKind) -> Result<Token, ()>
	{
		match self.maybe_expect(kind)
		{
			Some(token) => Ok(token),
			None =>
			{
				let descr = format!("expected {}", kind.printable());
				let span = self.tokens[self.index_prev].span.after();
				self.report.error_span(descr, &span);
				Err(())
			}
		}
	}
	
	
	pub fn expect_msg<S>(&mut self, kind: TokenKind, descr: S) -> Result<Token, ()>
	where S: Into<String>
	{
		match self.maybe_expect(kind)
		{
			Some(token) => Ok(token),
			None => Err(self.report.error_span(descr, &self.tokens[self.index_prev].span.after()))
		}
	}
	
	
	pub fn next_is_linebreak(&self) -> bool
	{
		self.read_linebreak || self.is_over()
	}
	
	
	pub fn maybe_expect_linebreak(&mut self) -> Option<()>
	{
		if self.next_is_linebreak()
		{
			self.read_linebreak = false;
			Some(())
		}
		else
			{ None }
	}
	
	
	pub fn expect_linebreak(&mut self) -> Result<(), ()>
	{
		if self.maybe_expect_linebreak().is_some()
			{ Ok(()) }
		else
			{ Err(self.report.error_span("expected line break", &self.tokens[self.index_prev].span.after())) }
	}
	
	
	pub fn expect_usize(&mut self) -> Result<(Token, usize), ()>
	{
		let tk = self.expect(TokenKind::Number)?;
		let value = excerpt_as_usize(self.report.clone(), &tk.excerpt.as_ref().unwrap(), &tk.span)?;
		Ok((tk, value))
	}
}