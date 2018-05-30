use diagn::{Span, RcReport};
use num_bigint::BigInt;
use num_traits::Zero;


pub fn excerpt_as_string_contents(report: RcReport, excerpt: &str, span: &Span) -> Result<String, ()>
{
	assert!(excerpt.len() >= 2);
	
	let mut chars = excerpt[1..(excerpt.len() - 1)].chars().peekable();
	
	let mut result = String::new();
	
	while let Some(c) = chars.peek().map(|c| *c)
	{
		chars.next();
		
		let unescaped = if c == '\\'
		{
			match chars.next()
			{
				Some('0')  => '\0',
				Some('t')  => '\t',
				Some('r')  => '\r',
				Some('n')  => '\n',
				Some('\'') => '\'',
				Some('\"') => '\"', // "
				Some('\\') => '\\',
				
				Some('x') =>
				{
					let mut byte = 0u8;
					
					for _ in 0..2
					{
						byte <<= 4;
						
						byte += match chars.next().map(|c| c.to_digit(16))
						{
							Some(Some(d)) => d as u8,
							_ => return Err(report.error_span("invalid escape sequence", span))
						};
					}
					
					if byte > 0x7f
						{ return Err(report.error_span("invalid escape sequence", span)); }
					
					byte as char
				}
				
				Some('u') =>
				{
					let mut codepoint = 0u32;
					
					if chars.next() != Some('{')
						{ return Err(report.error_span("invalid escape sequence", span)); }
					
					let mut i = 0;
					loop
					{
						if i > 6
							{ return Err(report.error_span("invalid escape sequence", span)); }
							
						i += 1;
						
						let digit = match chars.next()
						{
							Some('}') => break,
							Some(c) => match c.to_digit(16)
							{
								Some(d) => d,
								None => return Err(report.error_span("invalid escape sequence", span))
							}
							
							None => return Err(report.error_span("invalid escape sequence", span))
						};
						
						codepoint <<= 4;						
						codepoint += digit;
					}
					
					use std::char;
					match char::from_u32(codepoint)
					{
						Some(c) => c,
						None => return Err(report.error_span("invalid escape sequence", span))
					}
				}
				
				Some(_) |
				None => return Err(report.error_span("invalid escape sequence", span))
			}
		}
		else
			{ c };
			
		result.push(unescaped);
	}
	
	Ok(result)
}



pub fn excerpt_as_usize(report: RcReport, excerpt: &str, span: &Span) -> Result<usize, ()>
{
	let chars: Vec<char> = excerpt.chars().collect();
	assert!(chars.len() >= 1);

	let (radix, mut index) = parse_radix(&chars, 0);
	
	let mut value: usize = 0;
	while index < chars.len()
	{
		let c = chars[index];
		index += 1;
		
		if c == '_'
			{ continue; }
		
		let digit = match c.to_digit(radix as u32)
		{
			Some(d) => d,
			None => return Err(report.error_span("invalid digits", span))
		};
		
		value = match value.checked_mul(radix)
		{
			Some(v) => v,
			None => return Err(report.error_span("value is too large", span))
		};
		
		value = match value.checked_add(digit as usize)
		{
			Some(v) => v,
			None => return Err(report.error_span("value is too large", span))
		};
	}
	
	Ok(value)
}


pub fn excerpt_as_bigint(report: RcReport, excerpt: &str, span: &Span) -> Result<(BigInt, Option<usize>, usize, usize), ()>
{
	let chars: Vec<char> = excerpt.chars().collect();
	assert!(chars.len() >= 1);

	let (width,     index) = parse_width(report.clone(), &chars, span)?;
	let (radix, mut index) = parse_radix(&chars, index);
	
	let mut digit_num = 0;
	
	let mut value = BigInt::zero();
	while index < chars.len()
	{
		let c = chars[index];
		index += 1;
		
		if c == '_'
			{ continue; }
		
		let digit = match c.to_digit(radix as u32)
		{
			Some(d) => d,
			None => return Err(report.error_span("invalid digits", span))
		};
		
		digit_num += 1;
		
		value = value * radix;
		value = value + digit;
	}
	
	if let Some(width) = width
	{
		if value.bits() > width
			{ return Err(report.error_span(format!("value (width = {}) is larger than specified", value.bits()), span)); }
	}
	
	Ok((value, width, radix, digit_num))
}


fn parse_width(report: RcReport, chars: &[char], span: &Span) -> Result<(Option<usize>, usize), ()>
{
	if !chars.iter().any(|c| *c == '\'')
		{ return Ok((None, 0)); }

	let mut width: usize = 0;
	let mut index = 0;
	loop
	{
		let c = chars[index];
		index += 1;
		
		if c == '_'
			{ continue; }
			
		if c == '\''
			{ break; }
		
		let digit = match c.to_digit(10)
		{
			Some(d) => d,
			None => return Err(report.error_span("invalid digits in width specifier", span))
		};
		
		width = match width.checked_mul(10)
		{
			Some(v) => v,
			None => return Err(report.error_span("width specifier is too large", span))
		};
		
		width = match width.checked_add(digit as usize)
		{
			Some(v) => v,
			None => return Err(report.error_span("width specifier is too large", span))
		};
	}
	
	if width == 0
		{ return Err(report.error_span("invalid width specifier", span)); }
	
	Ok((Some(width), index))
}


fn parse_radix(chars: &[char], index: usize) -> (usize, usize)
{
	if chars[index] == '0' && index + 1 < chars.len()
	{
		match chars[index + 1]
		{
			'b' => ( 2, index + 2),
			'o' => ( 8, index + 2),
			'x' => (16, index + 2),
			_ =>   (10, index)
		}
	}
	else
		{ (10, index) }
}