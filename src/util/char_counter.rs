pub struct CharCounter<'s>
{
	chars: &'s [char]
}


impl<'s> CharCounter<'s>
{
	pub fn new(chars: &'s [char]) -> CharCounter<'s>
	{
		CharCounter
		{
			chars: chars
		}
	}
	
	
	pub fn get_excerpt(&self, start: usize, end: usize) -> &[char]
	{
		&self.chars[start..end]
	}
	
	
	pub fn get_line_count(&self) -> usize
	{
		let mut lines = 1;
		
		for &c in self.chars
		{
			if c == '\n'
				{ lines += 1; }
		}
		
		lines
	}
	
	
	pub fn get_line_column_at_index(&self, index: usize) -> (usize, usize)
	{
		let mut line = 0;
		let mut column = 0;
		
		let mut i = 0;
		while i < index && i < self.chars.len()
		{
			if self.chars[i] == '\n'
			{
				line += 1;
				column = 0;
			}
			else
				{ column += 1; }
			
			i += 1;
		}
		
		(line, column)
	}
	
	
	pub fn get_index_range_of_line(&self, line: usize) -> (usize, usize)
	{
		let mut line_count = 0;
		let mut line_begin = 0;
		
		while line_count < line && line_begin < self.chars.len()
		{
			line_begin += 1;
			
			if self.chars[line_begin - 1] == '\n'
				{ line_count += 1; }
		}
		
		let mut line_end = line_begin;
		while line_end < self.chars.len()
		{
			line_end += 1;
			
			if self.chars[line_end - 1] == '\n'
				{ break; }
		}
		
		(line_begin, line_end)
	}
}