use crate::*;


pub trait FileServer
{
	fn get_bytes(
		&self,
		report: &mut diagn::Report,
		span: Option<&diagn::Span>,
		filename: &str)
		-> Result<Vec<u8>, ()>;
	
	
	fn get_chars(
		&self,
		report: &mut diagn::Report,
		span: Option<&diagn::Span>,
		filename: &str)
		-> Result<Vec<char>, ()>
	{
		let bytes = self.get_bytes(
			report,
			span,
			filename)?;

		let string = String::from_utf8_lossy(&bytes)
			.chars()
			.collect();
		
		Ok(string)
	}
	
	
	fn write_bytes(
		&mut self,
		report: &mut diagn::Report,
		span: Option<&diagn::Span>,
		filename: &str,
		data: &Vec<u8>)
		-> Result<(), ()>;
	
	
	fn get_excerpt(
		&self,
		span: &diagn::Span)
		-> String
	{
		if let Ok(chars) = self.get_chars(
			&mut diagn::Report::new(),
			None,
			&*span.file)
		{
			let counter = util::CharCounter::new(&chars);
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
	files: std::collections::HashMap<String, Vec<u8>>
}


pub struct FileServerReal;


impl FileServerMock
{
	pub fn new() -> FileServerMock
	{
		FileServerMock
		{
			files: std::collections::HashMap::new()
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
	fn get_bytes(
		&self,
		report: &mut diagn::Report,
		span: Option<&diagn::Span>,
		filename: &str)
		-> Result<Vec<u8>, ()>
	{
		match self.files.get(filename)
		{
			Some(bytes) => Ok(bytes.clone()),

			None =>
			{
				report_error(
					report,
					span,
					format!(
						"file not found: `{}`",
						filename));

				return Err(());
			}
		}
	}
	
	
	fn write_bytes(
		&mut self,
		_report: &mut diagn::Report,
		_span: Option<&diagn::Span>,
		filename: &str,
		data: &Vec<u8>)
		-> Result<(), ()>
	{
		self.files.insert(filename.to_string(), data.clone());
		Ok(())
	}
}


impl FileServer for FileServerReal
{
	fn get_bytes(
		&self,
		report: &mut diagn::Report,
		span: Option<&diagn::Span>,
		filename: &str)
		-> Result<Vec<u8>, ()>
	{
		let filename_path = &std::path::Path::new(filename);
		
		if !filename_path.exists()
		{
			report_error(
				report,
				span,
				format!(
					"file not found: `{}`",
					filename));
			
			return Err(());
		}
		
		let mut file = {
			match std::fs::File::open(filename_path)
			{
				Ok(file) => file,
				Err(err) =>
				{
					report_error(
						report,
						span,
						format!(
							"could not open file `{}`: {}",
							filename,
							err));
					
					return Err(());
				}
			}
		};

		let mut vec = Vec::new();

		use std::io::Read;
		match file.read_to_end(&mut vec)
		{
			Ok(_) => Ok(vec),
			Err(err) =>
			{
				report_error(
					report,
					span,
					format!(
						"could not read file `{}`: {}",
						filename,
						err));
				
				return Err(());
			}
		}
	}
	
	
	fn write_bytes(
		&mut self,
		report: &mut diagn::Report,
		span: Option<&diagn::Span>,
		filename: &str,
		data: &Vec<u8>)
		-> Result<(), ()>
	{
		let filename_path = &std::path::Path::new(filename);
		
		let mut file = {
			match std::fs::File::create(filename_path)
			{
				Ok(file) => file,
				Err(err) =>
				{
					report_error(
						report,
						span,
						format!(
							"could not create file `{}`: {}",
							filename,
							err));

					return Err(());
				}
			}
		};

		use std::io::Write;
		match file.write_all(data)
		{
			Ok(_) => Ok(()),
			Err(err) => 
			{
				report_error(
					report,
					span,
					format!("could not write to file `{}`: {}",
						filename,
						err));

				Err(())
			}
		}
	}
}


fn report_error<S>(
	report: &mut diagn::Report,
	span: Option<&diagn::Span>,
	descr: S)
	where S: Into<String>
{
	if let Some(span) = span
	{
		report.error_span(descr, span);
	}
	else
	{
		report.error(descr);
	}
}