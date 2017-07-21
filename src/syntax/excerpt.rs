use diagn::{Span, Message};


pub fn excerpt_as_usize(excerpt: &str, span: &Span) -> Result<usize, Message>
{
	let chars: Vec<char> = excerpt.chars().collect();
	assert!(chars.len() >= 1);

	let mut index = 0;
	
	let radix: usize =
		if chars[index] == '0' && index + 1 < chars.len()
		{
			match chars[index + 1]
			{
				'b' => { index += 2;  2 }
				'x' => { index += 2; 16 }
				_ => 10
			}
		}
		else
			{ 10 };
	
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
			None => return Err(Message::error_span("invalid digits", span))
		};
		
		value = match value.checked_mul(radix)
		{
			Some(v) => v,
			None => return Err(Message::error_span("value is too large", span))
		};
		
		value = match value.checked_add(digit as usize)
		{
			Some(v) => v,
			None => return Err(Message::error_span("value is too large", span))
		};
	}
	
	Ok(value)
}