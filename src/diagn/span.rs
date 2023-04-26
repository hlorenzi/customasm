use crate::*;


pub type SpanIndex = usize;


#[derive(Copy, Clone, Hash, Eq)]
pub struct Span
{
	pub file_handle: util::FileServerHandle,
	pub location: Option<(SpanIndex, SpanIndex)>
}


impl Span
{
	pub fn new(
		file_handle: util::FileServerHandle,
		start: SpanIndex,
		end: SpanIndex)
		-> Span
	{
		Span {
			file_handle,
			location: Some((start, end))
		}
	}
	
	
	pub fn new_dummy() -> Span
	{
		Span {
			file_handle: 0,
			location: None,
		}
	}
	
	
	pub fn before(&self) -> Span
	{
		if self.location.is_none()
		{
			*self
		}
		
		else
		{
			let start = self.location.unwrap().0;
			
			Span {
				file_handle: self.file_handle,
				location: Some((start, start))
			}
		}
	}
	
	
	pub fn after(&self) -> Span
	{
		if self.location.is_none()
		{
			*self
		}
		
		else
		{
			let end = self.location.unwrap().1;
			
			Span {
				file_handle: self.file_handle,
				location: Some((end, end)),
			}
		}
	}
	
	
	pub fn join(&self, other: Span) -> Span
	{
		match (self.location, other.location)
		{
			(_, None) => *self,
			(None, _) => other,
			(Some(self_loc), Some(other_loc)) =>
			{
				assert!(
					self.file_handle == other.file_handle,
					"joining spans from different files");

				let location = {
					use std::cmp::{max, min};
					let start = min(self_loc.0, other_loc.0);
					let end   = max(self_loc.1, other_loc.1);
					Some((start, end))
				};
				
				Span {
					file_handle: self.file_handle,
					location,
				}
			}
		}
	}
}


impl PartialEq for Span
{
	fn eq(&self, other: &Self) -> bool
	{
		self.file_handle == other.file_handle && self.location == other.location
	}
}


impl std::fmt::Debug for Span
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("Span(")?;
		write!(f, "file#{:?}", &self.file_handle)?;

		if let Some(location) = self.location
		{
			f.write_str("[")?;
			<SpanIndex as std::fmt::Debug>::fmt(&location.0, f)?;
			f.write_str("..")?;
			<SpanIndex as std::fmt::Debug>::fmt(&location.1, f)?;
			f.write_str("]")?;
		}

        f.write_str(")")?;
        Ok(())
    }
}