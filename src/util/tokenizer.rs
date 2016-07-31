#[derive(Debug, Copy, Clone)]
pub struct Span
{
	pub start: usize,
	pub end: usize
}


#[derive(Debug, Clone)]
pub struct Token
{
	pub span: Span,
	pub kind: TokenKind
}


#[derive(Debug, Clone)]
pub enum TokenKind
{
	Error(char),
	LineBreak,
	Identifier(String),
	Number(usize, String),
	Operator(&'static str),
	End
}


pub fn tokenize(src: &[char]) -> Vec<Token>
{
	let mut tokens = Vec::new();
	let mut index = 0;
	let mut last_was_linebreak = true;
	
	while index < src.len()
	{
		if is_whitespace(src[index]) ||
			(last_was_linebreak && src[index] == '\n')
		{
			index += 1;
			continue;
		}
	
		let token =
			try_read_identifier(src, &mut index).unwrap_or_else(||
			try_read_integer(src, &mut index).unwrap_or_else(||
			try_read_linebreak(src, &mut index).unwrap_or_else(||
			try_read_operator(src, &mut index).unwrap_or_else(||
			read_error(src, &mut index)))));
		
		last_was_linebreak = match token.kind
		{
			TokenKind::LineBreak => true,
			_ => false
		};
		
		tokens.push(token);
	}
	
	tokens
}


impl Span
{
	pub fn get_line_column(&self, src: &[char]) -> (usize, usize)
	{
		let mut index = 0;
		let mut line = 1;
		let mut column = 1;
		
		while index < src.len() && index < self.start
		{
			if src[index] == '\n'
			{
				line += 1;
				column = 1;
			}
			else
			{
				column += 1;
			}
			
			index += 1;
		}
		
		(line, column)
	}
}


impl Token
{
	pub fn is_linebreak(&self) -> bool
	{
		match self.kind
		{
			TokenKind::LineBreak => true,
			_ => false
		}
	}
	
	
	pub fn is_linebreak_or_end(&self) -> bool
	{
		match self.kind
		{
			TokenKind::End => true,
			TokenKind::LineBreak => true,
			_ => false
		}
	}
	
	
	pub fn is_identifier(&self) -> bool
	{
		match self.kind
		{
			TokenKind::Identifier(..) => true,
			_ => false
		}
	}
	
	
	pub fn identifier(&self) -> &String
	{
		match self.kind
		{
			TokenKind::Identifier(ref ident) => &ident,
			_ => panic!("not an identifier")
		}
	}
	
	
	pub fn is_number(&self) -> bool
	{
		match self.kind
		{
			TokenKind::Number(..) => true,
			_ => false
		}
	}
	
	
	pub fn number(&self) -> (usize, &String)
	{
		match self.kind
		{
			TokenKind::Number(radix, ref value) => (radix, &value),
			_ => panic!("not a number")
		}
	}
	
	
	pub fn number_usize(&self) -> usize
	{
		match self.kind
		{
			TokenKind::Number(radix, ref value) => usize::from_str_radix(&value, radix as _).unwrap(),
			_ => panic!("not a number")
		}
	}
	
	
	pub fn is_any_operator(&self) -> bool
	{
		match self.kind
		{
			TokenKind::Operator(..) => true,
			_ => false
		}
	}
	
	
	pub fn is_operator(&self, op: &str) -> bool
	{
		match self.kind
		{
			TokenKind::Operator(token_op) => token_op == op,
			_ => false
		}
	}
	
	
	pub fn operator(&self) -> &'static str
	{
		match self.kind
		{
			TokenKind::Operator(op) => op,
			_ => panic!("not an operator")
		}
	}
}


fn try_read_identifier(src: &[char], index: &mut usize) -> Option<Token>
{
	let span_start = *index;
	
	if !is_identifier_start(src[*index])
		{ return None; }

	let mut identifier = String::new();
	while *index < src.len() && is_identifier_mid(src[*index])
	{
		identifier.push(src[*index]);
		*index += 1;
	}
	
	Some(Token
	{
		span: Span { start: span_start, end: *index },
		kind: TokenKind::Identifier(identifier)
	})
}


fn try_read_integer(src: &[char], index: &mut usize) -> Option<Token>
{
	let span_start = *index;
	
	if !src[*index].is_digit(10)
		{ return None; }

	let radix =
		if src[*index] == '0' && *index + 1 < src.len()
		{
			match src[*index + 1]
			{
				'b' => { *index += 2; 2 }
				'x' => { *index += 2; 16 }
				_ => 10
			}
		}
		else
			{ 10 };
	
	let mut digits = String::new();
	while *index < src.len()
	{
		if !src[*index].is_digit(radix) && src[*index] != '_'
			{ break; }
		
		digits.push(src[*index]);
		*index += 1;
	}
	
	Some(Token
	{
		span: Span { start: span_start, end: *index },
		kind: TokenKind::Number(radix as usize, digits)
	})
}


fn try_read_operator(src: &[char], index: &mut usize) -> Option<Token>
{
	let operators =
	[
		".", "->", ":", ";", ",",
		"(", ")", "[", "]", "{", "}",
		"'",
		"#", "$"
	];
	
	let maybe_match = operators.iter().find(|op|
	{
		for (i, c) in op.chars().enumerate()
		{
			if *index + i >= src.len() || src[*index + i] != c
				{ return false; }
		}
		true
	});
	
	match maybe_match
	{
		Some(s) =>
		{
			let len = s.chars().count();
			*index += len;
			Some(Token
			{
				span: Span { start: *index - len, end: *index },
				kind: TokenKind::Operator(s)
			})
		}
		None => None
	}
}


fn try_read_linebreak(src: &[char], index: &mut usize) -> Option<Token>
{
	if src[*index] != '\n'
		{ return None; }

	*index += 1;
	Some(Token
	{
		span: Span { start: *index - 1, end: *index },
		kind: TokenKind::LineBreak
	})
}


fn read_error(src: &[char], index: &mut usize) -> Token
{
	*index += 1;
	Token
	{
		span: Span { start: *index - 1, end: *index },
		kind: TokenKind::Error(src[*index - 1])
	}
}


fn is_whitespace(c: char) -> bool
{
	c == ' ' ||
	c == '\t' ||
	c == '\r'
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