use util::tokenizer::Span;


#[derive(Debug)]
pub struct Error
{
	pub msg: String,
	pub span: Option<Span>
}


impl Error
{
	pub fn new<S>(msg: S) -> Error
	where S: Into<String>
	{
		Error
		{
			msg: msg.into(),
			span: None
		}
	}
	
	
	pub fn new_with_span<S>(msg: S, span: Span) -> Error
	where S: Into<String>
	{
		Error
		{
			msg: msg.into(),
			span: Some(span)
		}
	}
	
	
	pub fn get_msg(&self) -> &String
	{
		&self.msg
	}
	
	
	pub fn get_line(&self) -> usize
	{
		match self.span
		{
			Some(ref span) => span.start.line,
			None => 0
		}
	}
	
	
	pub fn contains_str(&self, s: &str) -> bool
	{
		self.msg.find(s).is_some()
	}
	
	
	pub fn line_is(&self, line: usize) -> bool
	{
		match self.span
		{
			Some(ref span) => span.start.line == line,
			None => false
		}
	}
	
	
	pub fn print(&self)
	{
		if let Some(ref span) = self.span
		{
			print!("{}:{}:{}: {}:{} ",
				span.file,
				span.start.line, span.start.column,
				span.end.line, span.end.column);
		}
		
		println!("error: {}", self.msg);
	}
}