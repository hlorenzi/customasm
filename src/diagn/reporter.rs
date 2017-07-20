use diagn::Span;


pub struct Message
{
	pub descr: String,
	pub span: Option<Span>
}


impl Message
{
	pub fn error<S>(descr: S) -> Message
	where S: Into<String>
	{
		Message { descr: descr.into(), span: None }
	}
	
	
	pub fn error_span<S>(descr: S, span: &Span) -> Message
	where S: Into<String>
	{
		Message { descr: descr.into(), span: Some(span.clone()) }
	}
}


pub struct Reporter
{
	messages: Vec<Message>
}


impl Reporter
{
	pub fn new() -> Reporter
	{
		Reporter
		{
			messages: Vec::new()
		}
	}
	
	
	pub fn message(&mut self, msg: Message)
	{
		self.messages.push(msg);
	}
	
	
	pub fn error<S>(&mut self, descr: S)
	where S: Into<String>
	{
		self.message(Message::error(descr));
	}
	
	
	pub fn error_span<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.message(Message::error_span(descr, span));
	}
	
	
	pub fn print(&self)
	{
		for msg in &self.messages
		{
			println!("error: {}", msg.descr);
		}
	}
}