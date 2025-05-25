use crate::*;


pub fn excerpt_as_string_contents(
	report: &mut diagn::Report,
	span: diagn::Span,
	excerpt: &str)
	-> Result<String, ()>
{
	assert!(excerpt.len() >= 2);
	
	let without_quotes = &excerpt[1..(excerpt.len() - 1)];

	unescape_string(report, span, without_quotes)
}



pub fn unescape_string(
	report: &mut diagn::Report,
	span: diagn::Span,
	original_str: &str)
	-> Result<String, ()>
{
	let mut result = String::new();

	let mut chars = original_str.chars().peekable();

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
				Some('\"') => '\"',
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



pub fn excerpt_as_usize(
	report: &mut diagn::Report,
	span: diagn::Span,
	excerpt: &str)
	-> Result<usize, ()>
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
			None => 
			{
				report.error_span(
					"invalid digits",
					span);
				
				return Err(());
			}
		};
		
		value = match value.checked_mul(radix)
		{
			Some(v) => v,
			None =>
			{
				report.error_span(
					"value is too large",
					span);
				
				return Err(());
			}
		};
		
		value = match value.checked_add(digit as usize)
		{
			Some(v) => v,
			None =>
			{
				report.error_span(
					"value is too large",
					span);
				
				return Err(());
			}
		};
	}
	
	Ok(value)
}


pub fn excerpt_as_bigint(
	report: Option<&mut diagn::Report>,
	span: diagn::Span,
	excerpt: &str)
	-> Result<util::BigInt, ()>
{
	let chars: Vec<char> = excerpt.chars().collect();
	assert!(chars.len() >= 1);

	let (radix, mut index) = parse_radix(&chars, 0);
	
	let mut digit_num = 0;
	
	let mut value = num_bigint::BigInt::from(0);
	while index < chars.len()
	{
		let c = chars[index];
		index += 1;
		
		if c == '_'
			{ continue; }
		
		let digit = match c.to_digit(radix as u32)
		{
			Some(d) => d,
			None => 
			{
				if let Some(report) = report
				{
					report.error_span(
						"invalid digits",
						span);
				}
				
				return Err(());
			}
		};
		
		digit_num += 1;
		
		value = value * radix;
		value = value + digit;
	}
	
	if digit_num == 0
	{
		if let Some(report) = report
		{
			report.error_span(
				"invalid value",
				span);
		}
		
		return Err(());
	}
		
	let radix_bits = match radix
	{
		2 => Some(1),
		8 => Some(3),
		16 => Some(4),
		_ => None
	};

	let size = match radix_bits
	{
		None => None,
		Some(radix_bits) => Some(radix_bits * digit_num)
	};
	
	Ok(util::BigInt::new(value, size))
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
	{
		match chars[0]
		{
			'%' => ( 2, index + 1),
			'$' => (16, index + 1),
			_ =>   (10, index)
		}
	}
}