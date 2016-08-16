use util::tokenizer::Span;


#[derive(Debug)]
pub struct Error
{
	pub msg: String,
	pub span: Option<Span>
}


pub fn handle_opt_span<T, S>(option: Option<T>, msg: S, span: &Span) -> Result<T, Error>
where S: Into<String>
{
    match option
	{
        Some(val) => Ok(val),
		
        None =>
            return Err(Error::new_with_span(msg.into(), span.clone()))
    }
}


pub fn handle_result_span<T>(result: Result<T, Error>, span: &Span) -> Result<T, Error>
{
    match result
	{
        Ok(val) => Ok(val),
		
        Err(mut err) =>
		{
			if err.span.is_none()
				{ err.span = Some(span.clone()); }
				
            return Err(err);
		}
    }
}


pub fn handle_result_msg_span<T, S>(result: Result<T, Error>, msg: S, span: &Span) -> Result<T, Error>
where S: Into<String>
{
    match result
	{
        Ok(val) => Ok(val),
		
        Err(mut err) =>
		{
			err.msg = msg.into();
		
			if err.span.is_none()
				{ err.span = Some(span.clone()); }
				
            return Err(err);
		}
    }
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
	
	
	pub fn get_file(&self) -> &str
	{
		match self.span
		{
			Some(ref span) => span.file.as_ref(),
			None => panic!("no filename")
		}
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
	
	
	pub fn file_is(&self, file: &str) -> bool
	{
		match self.span
		{
			Some(ref span) => span.file.as_ref() == file,
			None => false
		}
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