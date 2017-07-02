use std::rc::Rc;


#[derive(Debug, Clone)]
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
	
	
	pub fn join(&self, other: &Span) -> Span
	{
		assert!(self.file == other.file, "joining spans from different files");
		
		
		let location = if self.location.is_none()
			{ other.location }
			
		else if other.location.is_none()
			{ self.location }
			
		else
		{
			use std::cmp::{max, min};
			let start = min(self.location.unwrap().0, other.location.unwrap().0);
			let end   = max(self.location.unwrap().1, other.location.unwrap().1);
			Some((start, end))
		};
		
		
		Span
		{
			file: self.file.clone(),
			location: location
		}
	}
}