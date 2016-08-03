use util::tokenizer::Span;


#[derive(Debug)]
pub struct Error
{
	msg: String,
	filename: Option<String>,
	span: Option<Span>
}


impl Error
{
	pub fn new<S>(msg: S) -> Error
	where S: Into<String>
	{
		Error
		{
			msg: msg.into(),
			filename: None,
			span: None
		}
	}
	
	
	pub fn new_with_file<S, T>(msg: S, filename: T) -> Error
	where S: Into<String>, T: Into<String>
	{
		Error
		{
			msg: msg.into(),
			filename: Some(filename.into()),
			span: None
		}
	}
	
	
	pub fn new_with_span<S>(msg: S, span: Span) -> Error
	where S: Into<String>
	{
		Error
		{
			msg: msg.into(),
			filename: None,
			span: Some(span)
		}
	}
	
	
	pub fn new_with_file_span<S, T>(msg: S, filename: T, span: Span) -> Error
	where S: Into<String>, T: Into<String>
	{
		Error
		{
			msg: msg.into(),
			filename: Some(filename.into()),
			span: Some(span)
		}
	}
	
	
	pub fn set_file<S>(&mut self, filename: S)
	where S: Into<String>
	{
		self.filename = Some(filename.into());
	}
	
	
	pub fn print(&self)
	{
		if let Some(ref filename) = self.filename
		{
			print!("{}: ", filename);
		}
	
		if let Some(ref span) = self.span
		{
			print!("{}: {}:{}: {}:{} ",
				span.file,
				span.start.line, span.start.column,
				span.end.line, span.end.column);
		}
		
		println!("error: {}", self.msg);
	}
}