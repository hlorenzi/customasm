use diagn::{Span, Report};
use std::collections::HashMap;


pub trait FileServer
{
	fn get_bytes(&self, report: &mut Report, filename: &str, span: Option<&Span>) -> Result<Vec<u8>, ()>;
	
	
	fn get_chars(&self, report: &mut Report, filename: &str, span: Option<&Span>) -> Result<Vec<char>, ()>
	{
		let bytes = self.get_bytes(report, filename, span)?;
		
		Ok(String::from_utf8_lossy(&bytes).chars().collect())
	}
}


pub struct FileServerMock
{
	files: HashMap<String, Vec<u8>>
}


impl FileServerMock
{
	pub fn new() -> FileServerMock
	{
		FileServerMock
		{
			files: HashMap::new()
		}
	}
	
	
	pub fn add<S, T>(&mut self, filename: S, contents: T)
	where S: Into<String>, T: Into<Vec<u8>>
	{
		self.files.insert(filename.into(), contents.into());
	}
}


impl FileServer for FileServerMock
{
	fn get_bytes(&self, report: &mut Report, filename: &str, span: Option<&Span>) -> Result<Vec<u8>, ()>
	{
		match self.files.get(filename)
		{
			None =>
			{
				let descr = format!("file not found: `{}`", filename);
				if let Some(span) = span
					{ report.error_span(descr, span); }
				else
					{ report.error(descr); }
					
				Err(())
			}
			Some(bytes) => Ok(bytes.clone())
		}
	}
}