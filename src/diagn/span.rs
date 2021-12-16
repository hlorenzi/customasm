use std::rc::Rc;


#[derive(Debug, Clone, Hash, Eq)]
pub struct Span
{
	pub file: Rc<String>,
	pub location: Option<(usize, usize)>
}


impl Span
{
	pub fn new(filename: Rc<String>, start: usize, end: usize) -> Span
	{
		Span
		{
			file: filename,
			location: Some((start, end))
		}
	}
	
	
	pub fn new_dummy() -> Span
	{
		Span
		{
			file: Rc::new("".to_string()),
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

		Span
		{
			file: self.file.clone(),
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