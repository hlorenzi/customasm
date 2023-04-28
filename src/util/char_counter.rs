use crate::*;


pub struct CharCounter
{
	chars: Vec<char>,
}


impl CharCounter
{
	pub fn new(src: &str) -> CharCounter
	{
		CharCounter
		{
			chars: src.chars().collect(),
		}
	}
	
	
	pub fn get_excerpt(
		&self,
		start: diagn::SpanIndex,
		end: diagn::SpanIndex)
		-> &[char]
	{
		&self.chars[(start as usize)..(end as usize)]
	}
	
	
	pub fn get_line_count(&self) -> usize
	{
		let mut lines = 1;
		
		for c in &self.chars
		{
			if *c == '\n'
				{ lines += 1; }
		}
		
		lines
	}
	
	
	pub fn get_line_column_at_index(
		&self,
		index: diagn::SpanIndex)
		-> (usize, usize)
	{
		let mut line = 0;
		let mut column = 0;
		
		let mut i = 0;
		while i < index as usize && i < self.chars.len()
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
	
	
	pub fn get_index_range_of_line(
		&self,
		line: usize)
		-> (diagn::SpanIndex, diagn::SpanIndex)
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
		
		(
			line_begin.try_into().unwrap(),
			line_end.try_into().unwrap()
		)
	}
}