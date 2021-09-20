use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct Span
{
	pub file: Rc<String>,
	pub line: Option<usize>,
	pub location: Option<(usize, usize)>
}


impl Span
{
	pub fn new(filename: Rc<String>, line: usize, start: usize, end: usize) -> Span
	{
		Span
		{
			file: filename,
			line: Some(line),
			location: Some((start, end))
		}
	}
	
	
	pub fn new_dummy() -> Span
	{
		Span
		{
			file: Rc::new("".to_string()),
			line: None,
			location: None
		}
	}
	
	
	pub fn before(&self) -> Span
	{
		if self.location.is_none()
			{ self.clone() }
		
		else
		{
			let start = self.location.unwrap().0;
			
			Span
			{
				file: self.file.clone(),
				line: self.line,
				location: Some((start, start))
			}
		}
	}
	
	
	pub fn after(&self) -> Span
	{
		if self.location.is_none()
			{ self.clone() }
		
		else
		{
			let end = self.location.unwrap().1;
			
			Span
			{
				file: self.file.clone(),
				line: self.line,
				location: Some((end, end))
			}
		}
	}
	
	
	pub fn join(&self, other: &Span) -> Span
	{
		if self.location.is_none()
			{ return other.clone(); }
			
		else if other.location.is_none()
			{ return self.clone(); }
			
		assert!(self.file == other.file, "joining spans from different files");

		let location =
		{
			use std::cmp::{max, min};
			let start = min(self.location.unwrap().0, other.location.unwrap().0);
			let end   = max(self.location.unwrap().1, other.location.unwrap().1);
			Some((start, end))
		};

		let line = if self.location.unwrap().0 <= other.location.unwrap().0
		{
			self.line
		}
		else
		{
			other.line
		};
		
		Span
		{
			file: self.file.clone(),
			line,
			location
		}
	}
}


impl PartialEq for Span
{
	fn eq(&self, other: &Self) -> bool
	{
		self.file == other.file && self.location == other.location
	}
}