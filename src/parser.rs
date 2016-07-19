pub struct Parser<'src>
{
	src: &'src mut Iterator<Item = char>,
	lookahead: Vec<char>,
	line_num: usize,
	column_num: usize
}


#[derive(Debug)]
pub struct ParserError
{
	pub msg: String,
	pub line_num: usize,
	pub column_num: usize
}


impl<'src> Parser<'src>
{
	pub fn new(src: &'src mut Iterator<Item = char>) -> Parser<'src>
	{
		let mut parser = Parser
		{
			src: src,
			lookahead: Vec::new(),
			line_num: 1,
			column_num: 1
		};
		
		if let Some(next) = parser.src.next()
			{ parser.lookahead.push(next); }
		
		parser
	}
	
	
	fn current(&self) -> char
	{
		if self.lookahead.len() > 0
			{ self.lookahead[0] }
		else
			{ 0 as char }
	}
	
	
	fn next(&mut self, index: usize) -> char
	{
		while self.lookahead.len() < index + 1
		{
			if let Some(next) = self.src.next()
				{ self.lookahead.push(next); }
			else
				{ return 0 as char; }
		}
		
		return self.lookahead[index];
	}
	
	
	pub fn is_over(&self) -> bool
	{
		self.lookahead.len() == 0
	}
	
	
	pub fn advance(&mut self)
	{
		self.column_num += 1;
		if self.current() == '\n'
		{
			self.line_num += 1;
			self.column_num = 1;
		}
	
		if self.lookahead.len() > 0
			{ self.lookahead.remove(0); }
			
		if self.lookahead.len() == 0
		{
			if let Some(next) = self.src.next()
				{ self.lookahead.push(next); }
		}
	}
	
	
	pub fn skip_white(&mut self)
	{
		while is_whitespace(self.current())
		{
			self.advance();
		}
	}
	
	
	pub fn matches(&mut self, c: char) -> bool
	{
		if self.current() == c
		{
			self.advance();
			true
		}
		else
			{ false }
	}
	
	
	pub fn matches_str(&mut self, s: &str) -> bool
	{
		let mut index = 0;
		for c in s.chars()
		{
			if self.next(index) != c
				{ return false; }
				
			index += 1;
		}
		
		for _ in s.chars()
			{ self.advance(); }
		
		true
	}
	
	
	pub fn expect(&mut self, c: char) -> Result<(), ParserError>
	{
		if self.current() == c
		{
			self.advance();
			Ok(())
		}
		else
		{
			Err(self.error(format!("expected `{}`", c)))
		}
	}
	
	
	pub fn expect_str(&mut self, s: &str) -> Result<(), ParserError>
	{
		if !self.matches_str(s)
			{ Err(self.error(format!("expected `{}`", s))) }
		else
			{ Ok(()) }
	}
	
	
	pub fn current_is(&mut self, c: char) -> bool
	{
		self.current() == c
	}
	
	
	pub fn current_is_str(&mut self, s: &str) -> bool
	{
		let mut index = 0;
		for c in s.chars()
		{
			if self.next(index) != c
				{ return false; }
				
			index += 1;
		}
		
		true
	}
	
	
	pub fn current_is_pattern(&self) -> bool
	{
		is_pattern(self.current())
	}
	
	
	pub fn current_is_number(&self) -> bool
	{
		is_number_start(self.current())
	}
	
	
	pub fn get_pattern(&mut self) -> Result<String, ParserError>
	{
		if !is_pattern(self.current())
			{ return Err(self.error("expected pattern".to_string())); }
		
		let mut result = String::new();
		result.push(self.current());
		self.advance();
		
		while is_pattern(self.current())
		{
			result.push(self.current());
			self.advance();
		}
		
		Ok(result)
	}
	
	
	pub fn get_identifier(&mut self) -> Result<String, ParserError>
	{
		if !is_identifier_start(self.current())
			{ return Err(self.error("expected identifier".to_string())); }
		
		let mut result = String::new();
		result.push(self.current());
		self.advance();
		
		while is_identifier_mid(self.current())
		{
			result.push(self.current());
			self.advance();
		}
		
		Ok(result)
	}
	
	
	pub fn get_number(&mut self) -> Result<String, ParserError>
	{
		if !is_number(self.current())
			{ return Err(self.error("expected number".to_string())); }
		
		let mut result = String::new();
		result.push(self.current());
		self.advance();
		
		while is_number(self.current())
		{
			result.push(self.current());
			self.advance();
		}
		
		Ok(result)
	}
	
	
	pub fn get_integer(&mut self) -> Result<String, ParserError>
	{
		if !is_number_start(self.current())
			{ return Err(self.error("expected number".to_string())); }
		
		let mut result = String::new();
		result.push(self.current());
		self.advance();
		
		while is_number(self.current())
		{
			result.push(self.current());
			self.advance();
		}
		
		Ok(result)
	}
	
	
	pub fn get_radix_integer(&mut self) -> Result<(usize, String), ParserError>
	{
		let mut radix = 10;
		
		if self.matches_str("0x")
			{ radix = 16; }
		else if self.matches_str("0b")
			{ radix = 2; }
			
		let number = try!(self.get_number());
		Ok((radix, number))
	}
	
	
	pub fn get_bits(&mut self) -> Result<Vec<bool>, ParserError>
	{
		let mut bits = Vec::new();
		let mut radix = 10;
		let mut value_str = try!(self.get_integer());
		let mut bit_num = 0;
		
		if self.matches('\'')
		{
			bit_num = value_str.parse::<usize>().unwrap();
			let radix_integer = try!(self.get_radix_integer());
			radix = radix_integer.0;
			value_str = radix_integer.1;
		}
		
		if radix == 10
		{
			let mut value = value_str.parse::<usize>().unwrap();
			while bits.len() < bit_num
			{
				bits.insert(0, value & 1 != 0);
				value >>= 1;
			}
			
			if value != 0
				{ return Err(self.error(format!("value `{}` does not fit given size of `{}`", value_str, bit_num))); }
		}
		
		Ok(bits)
	}
	
	
	pub fn error(&self, msg: String) -> ParserError
	{
		ParserError
		{
			msg: msg,
			line_num: self.line_num,
			column_num: self.column_num
		}
	}
}


fn is_whitespace(c: char) -> bool
{
	c == ' ' ||
	c == '\t' ||
	c == '\r' ||
	c == '\n'
}


fn is_identifier_start(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	c == '_'
}


fn is_identifier_mid(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	(c >= '0' && c <= '9') ||
	c == '_'
}


fn is_number(c: char) -> bool
{
	c >= '0' && c <= '9'
}


fn is_number_start(c: char) -> bool
{
	(c >= '0' && c <= '9') ||
	c == '-' ||
	c == '+'
}


fn is_pattern(c: char) -> bool
{
	(c >= 'a' && c <= 'z') ||
	(c >= 'A' && c <= 'Z') ||
	(c >= '0' && c <= '9') ||
	c == '_' || c == '/' || c == '\\' ||
	c == '@' || c == '#' || c == '$' ||
	c == '%' || c == '^' || c == '&' || c == '*' ||
	c == '(' || c == ')' || c == '[' || c == ']' ||
	c == '<' || c == '>' || c == ':' || c == '?' ||
	c == '-' || c == '+' || c == '=' || c == '|' ||
	c == '~' || c == '`' || c == '\''
}