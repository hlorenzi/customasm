use diagn::{Span, Report};
use num::BigInt;
use num::Zero;


pub fn excerpt_as_string_contents(excerpt: &str) -> String
{
	let chars: Vec<char> = excerpt.chars().collect();
	assert!(chars.len() >= 2);
	
	chars[1..(chars.len() - 1)].iter().collect()
}



pub fn excerpt_as_usize(report: &mut Report, excerpt: &str, span: &Span) -> Result<usize, ()>
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


pub fn excerpt_as_bigint(report: &mut Report, excerpt: &str, span: &Span) -> Result<(BigInt, Option<usize>), ()>
{
	let chars: Vec<char> = excerpt.chars().collect();
	assert!(chars.len() >= 1);

	let (width,     index) = parse_width(report, &chars, span)?;
	let (radix, mut index) = parse_radix(&chars, index);
	
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
		
		value = value * radix;
		value = value + digit;
	}
	
	if let Some(width) = width
	{
		if value.bits() > width
			{ return Err(report.error_span(format!("value (width = {}) is larger than specified", value.bits()), span)); }
	}
	
	Ok((value, width))
}


fn parse_width(report: &mut Report, chars: &[char], span: &Span) -> Result<(Option<usize>, usize), ()>
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
			'x' => (16, index + 2),
			_ =>   (10, index)
		}
	}
	else
		{ (10, index) }
}