use crate::*;
use crate::diagn::RcReport;
use crate::syntax::{Token, TokenKind, excerpt_as_usize};


#[derive(Clone)]
pub struct Parser<'a>
{
	pub report: Option<RcReport>,
	tokens: &'a [Token],
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


impl<'a> Parser<'a>
{
	pub fn new(report: Option<RcReport>, tokens: &[Token]) -> Parser
	{
		//assert!(tokens[tokens.len() - 1].kind == TokenKind::End);
	
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


	pub fn suppress_reports(&mut self)
	{
		self.report = None;
	}


	pub fn get_current_token_index(&self) -> usize
	{
		self.index
	}


	pub fn get_previous_token_index(&self) -> usize
	{
		self.index_prev
	}


	pub fn get_full_span(&self) -> diagn::Span
	{
		self.tokens[0].span.join(&self.tokens.last().unwrap().span)
	}


	pub fn clone_slice(&self, start: usize, end: usize) -> Parser
	{
		Parser::new(self.report.clone(), &self.tokens[start..end])
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


	pub fn next_partial(&mut self) -> char
	{
		if self.tokens[self.index].kind == TokenKind::End
			{ return '\0'; }

		let sliced = unsafe { self.tokens[self.index].text().get_unchecked(self.partial_index..) };
		let mut char_indices = sliced.char_indices();
		char_indices.next().unwrap().1
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
				if let Some(ref report) = self.report
				{
					report.error_span(descr, &span);
				}
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
			None =>
			{
				if let Some(ref report) = self.report
				{
					report.error_span(descr, &self.tokens[self.index_prev].span.after());
				}
				Err(())
			}
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
		{
			if let Some(ref report) = self.report
			{
				report.error_span("expected line break", &self.tokens[self.index_prev].span.after());
			}
			Err(())
		}
	}
	
	
	pub fn expect_usize(&mut self) -> Result<(Token, usize), ()>
	{
		let tk = self.expect(TokenKind::Number)?;
		let value = excerpt_as_usize(self.report.clone(), &tk.excerpt.as_ref().unwrap(), &tk.span)?;
		Ok((tk, value))
	}
	
	
	pub fn maybe_expect_partial_usize(&mut self) -> Option<usize>
	{
		let mut value: usize = 0;
		let mut advance_count: usize = 0;

		while !self.is_over()
		{
			let c = self.next_partial();

			let digit = match c.to_digit(10)
			{
				Some(d) => d,
				None => break
			};
			
			value = match value.checked_mul(10)
			{
				Some(v) => v,
				None => break
			};
			
			value = match value.checked_add(digit as usize)
			{
				Some(v) => v,
				None => break
			};
			
			self.advance_partial();
			advance_count += 1;
		}

		if advance_count == 0
			{ return None; }

		Some(value)
	}
}