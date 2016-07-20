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
			{ Err(self.error(format!("expected `{}`", c))) }
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
	
	
	pub fn current_is_identifier(&self) -> bool
	{
		is_identifier_start(self.current())
	}
	
	
	pub fn current_is_number(&self) -> bool
	{
		self.current().is_digit(10)
	}
	
	
	pub fn get_line(&mut self) -> String
	{
		let mut result = String::new();
		
		while !self.is_over() && self.current() != '\n'
		{
			result.push(self.current());
			self.advance();
		}
		
		if self.current() == '\n'
			{ self.advance(); }

		result
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
	
	
	pub fn get_usize(&mut self) -> Result<usize, ParserError>
	{
		let radix =
			if self.matches_str("0x")
				{ 16 }
			else if self.matches_str("0b")
				{ 2 }
			else
				{ 10 };
			
		if !self.current().is_digit(16)
			{ return Err(self.error("expected number".to_string())); }
			
		if !self.current().is_digit(radix)
			{ return Err(self.error("invalid digit".to_string())); }
		
		let mut result = self.current().to_digit(radix).unwrap() as usize;
		self.advance();
		
		while self.current().is_digit(16)
		{
			if !self.current().is_digit(radix)
				{ return Err(self.error("invalid digit".to_string())); }
			
			let digit = self.current().to_digit(radix).unwrap() as usize;
			result = match result.checked_mul(radix as usize).and_then(|x| x.checked_add(digit))
			{
				Some(x) => x,
				None => return Err(self.error("value is too large".to_string()))
			};
			self.advance();
		}
		
		Ok(result)
	}
	
	
	pub fn get_bits(&mut self) -> Result<Vec<bool>, ParserError>
	{
		let bit_num = try!(self.get_usize());
		if bit_num == 0
			{ return Err(self.error("invalid bit length".to_string())); }
		
		try!(self.expect('\''));
		let (radix, value_str) = try!(self.get_integer_str());
		
		let mut bits = Vec::new();
		match radix
		{
			10 =>
			{
				let mut value = match value_str.parse::<u64>()
				{
					Ok(x) => x,
					Err(_) => return Err(self.error(format!("decimal value `{}` is too large", value_str)))
				};
				
				while value != 0
				{
					bits.insert(0, value & 1 != 0);
					value >>= 1;
				}
			}
			
			16 =>
			{
				for c in value_str.chars()
				{
					let mut nibble = c.to_digit(16).unwrap();
					for _ in 0..4
					{
						bits.push(nibble & 0b1000 != 0);
						nibble <<= 1;
					}
				}
			}
			
			2 =>
			{
				for c in value_str.chars()
					{ bits.push(c == '1'); }
			}
			
			_ => unreachable!()
		}
		
		while bits.len() > 1 && !bits[0]
			{ bits.remove(0); }
		
		if bits.len() > bit_num
			{ return Err(self.error(format!("value `{}{}` does not fit given size of `{}`", get_radix_prefix(radix), value_str, bit_num))); }
			
		while bits.len() < bit_num
			{ bits.insert(0, false); }
		
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
	
	
	fn get_integer_str(&mut self) -> Result<(usize, String), ParserError>
	{
		let radix =
			if self.matches_str("0x")
				{ 16 }
			else if self.matches_str("0b")
				{ 2 }
			else
				{ 10 };
			
		if !self.current().is_digit(16)
			{ return Err(self.error("expected number".to_string())); }
			
		if !self.current().is_digit(radix)
			{ return Err(self.error("invalid digit".to_string())); }
		
		let mut value = String::new();
		value.push(self.current());
		self.advance();
		
		while self.current().is_digit(16)
		{
			if !self.current().is_digit(radix)
				{ return Err(self.error("invalid digit".to_string())); }
			
			value.push(self.current());
			self.advance();
		}
		
		Ok((radix as usize, value))
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


fn get_radix_prefix(radix: usize) -> &'static str
{
	match radix
	{
		2 => "0b",
		10 => "",
		16 => "0x",
		_ => panic!("invalid radix")
	}
}