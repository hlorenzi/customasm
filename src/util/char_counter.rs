pub struct CharCounter<'a>
{
	src: &'a str,
	chars: Vec<(usize, char)>,
}


impl<'a> CharCounter<'a>
{
	pub fn new(src: &'a str) -> CharCounter<'a>
	{
		CharCounter {
			src,
			chars: src.char_indices().collect(),
		}
	}
	
	
	pub fn get_excerpt(
		&self,
		start: usize,
		end: usize)
		-> &str
	{
		&self.src[start..end]
	}
	
	
	pub fn get_line_count(&self) -> usize
	{
		let mut lines = 1;
		
		for c in &self.chars
		{
			if c.1 == '\n'
				{ lines += 1; }
		}
		
		lines
	}
	
	
	pub fn get_line_column_at_byte_index(
		&self,
		byte_index: usize)
		-> (usize, usize)
	{
		let mut line = 0;
		let mut column = 0;
		
		let mut char_index = 0;
		while char_index < self.chars.len()
		{
			if byte_index == self.chars[char_index].0
				{ break; }

			if self.chars[char_index].1 == '\n'
			{
				line += 1;
				column = 0;
			}
			else
				{ column += 1; }
			
			char_index += 1;
		}
		
		(line, column)
	}
	
	
	pub fn get_byte_range_of_line(
		&self,
		line: usize)
		-> (usize, usize)
	{
		let mut char_begin = 0;
		let mut byte_begin = 0;
		
		let mut line_count = 0;
		while line_count < line && char_begin < self.chars.len()
		{
			if self.chars[char_begin].1 == '\n'
				{ line_count += 1; }
			
			byte_begin = self.chars[char_begin].0 + self.chars[char_begin].1.len_utf8();
			char_begin += 1;
		}
		
		let mut char_end = char_begin;
		let mut byte_end = byte_begin;
		while char_end < self.chars.len()
		{
			if self.chars[char_end].1 == '\n'
				{ break; }

			byte_end = self.chars[char_end].0 + self.chars[char_end].1.len_utf8();
			char_end += 1;
		}
		
		(byte_begin, byte_end)
	}
}