use crate::diagn::{Span, RcReport};
use crate::util::CharCounter;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;


pub trait FileServer
{
	fn exists(&self, filename: &str) -> bool;


	fn get_bytes(&self, report: RcReport, filename: &str, span: Option<&Span>) -> Result<Vec<u8>, ()>;
	
	
	fn get_chars(&self, report: RcReport, filename: &str, span: Option<&Span>) -> Result<Vec<char>, ()>
	{
		let bytes = self.get_bytes(report, filename, span)?;
		
		Ok(String::from_utf8_lossy(&bytes).chars().collect())
	}
	
	
	fn write_bytes(&mut self, report: RcReport, filename: &str, data: &Vec<u8>, span: Option<&Span>) -> Result<(), ()>;
	
	
	fn get_excerpt(&self, span: &Span) -> String
	{
		if let Ok(chars) = self.get_chars(RcReport::new(), &*span.file, None)
		{
			let counter = CharCounter::new(&chars);
			let location = span.location.unwrap();
			counter.get_excerpt(location.0, location.1).iter().collect()
		}
		else
		{
			"".to_string()
		}
	}
}


pub struct FileServerMock
{
	files: HashMap<String, Vec<u8>>
}


pub struct FileServerReal;


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


impl FileServerReal
{
	pub fn new() -> FileServerReal
	{
		FileServerReal
	}
}


impl FileServer for FileServerMock
{
	fn exists(&self, filename: &str) -> bool
	{
		self.files.get(filename).is_some()
	}


	fn get_bytes(&self, report: RcReport, filename: &str, span: Option<&Span>) -> Result<Vec<u8>, ()>
	{
		match self.files.get(filename)
		{
			None => Err(error(report, format!("file not found: `{}`", filename), span)),
			Some(bytes) => Ok(bytes.clone())
		}
	}
	
	
	fn write_bytes(&mut self, _report: RcReport, filename: &str, data: &Vec<u8>, _span: Option<&Span>) -> Result<(), ()>
	{
		self.files.insert(filename.to_string(), data.clone());
		Ok(())
	}
}


impl FileServer for FileServerReal
{
	fn exists(&self, _filename: &str) -> bool
	{
		unimplemented!()
	}


	fn get_bytes(&self, report: RcReport, filename: &str, span: Option<&Span>) -> Result<Vec<u8>, ()>
	{
		let filename_path = &Path::new(filename);
		
		if !filename_path.exists()
			{ return Err(error(report, format!("file not found: `{}`", filename), span)); }
		
		let mut file = match File::open(filename_path)
		{
			Ok(file) => file,
			Err(err) => return Err(error(report, format!("could not open file `{}`: {}", filename, err), span))
		};

		let mut vec = Vec::new();
		match file.read_to_end(&mut vec)
		{
			Ok(_) => Ok(vec),
			Err(err) => Err(error(report, format!("could not read file `{}`: {}", filename, err), span))
		}
	}
	
	
	fn write_bytes(&mut self, report: RcReport, filename: &str, data: &Vec<u8>, span: Option<&Span>) -> Result<(), ()>
	{
		let filename_path = &Path::new(filename);
		
		let mut file = match File::create(filename_path)
		{
			Ok(file) => file,
			Err(err) => return Err(error(report, format!("could not create file `{}`: {}", filename, err), span))
		};

		match file.write_all(data)
		{
			Ok(_) => Ok(()),
			Err(err) => Err(error(report, format!("could not write to file `{}`: {}", filename, err), span))
		}
	}
}


fn error<S>(report: RcReport, descr: S, span: Option<&Span>)
where S: Into<String>
{
	if let Some(span) = span
		{ report.error_span(descr, span); }
	else
		{ report.error(descr); }
}