use diagn::Span;
use util::CharCounter;
use util::FileServer;
use util::enable_windows_ansi_support;
use std::rc::Rc;
use std::cell::RefCell;


const C_DEFAULT:  &'static str = "\u{1B}[0m";
const C_LOCATION: &'static str = "\u{1B}[0m\u{1B}[90m";
const C_ERROR:    &'static str = "\u{1B}[0m\u{1B}[91m";
const C_LINENUM:  &'static str = "\u{1B}[0m\u{1B}[90m";
const C_SRC:      &'static str = "\u{1B}[0m\u{1B}[97m"; //"


pub struct Report
{
	messages: Vec<Message>,
	parents: Vec<Message>
}


struct Message
{
	pub descr: String,
	pub span: Option<Span>,
	pub inner: Option<Box<Message>>
}


#[derive(Clone)]
pub struct RcReport
{
	report: Rc<RefCell<Report>>
}


pub struct ReportParentGuard
{
	report: RcReport
}


impl Report
{
	pub fn new() -> Report
	{
		Report
		{
			messages: Vec::new(),
			parents: Vec::new()
		}
	}
	
	
	fn message(&mut self, mut msg: Message)
	{
		for parent in self.parents.iter().rev()
			{ msg = Message{ descr: parent.descr.clone(), span: parent.span.clone(), inner: Some(Box::new(msg)) }; }
		
		self.messages.push(msg);
	}
	
	
	pub fn error<S>(&mut self, descr: S)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), span: None, inner: None };
		self.message(msg);
	}
	
	
	pub fn error_span<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), span: Some(span.clone()), inner: None };
		self.message(msg);
	}
	
	
	pub fn push_parent<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), span: Some(span.clone()), inner: None };
		self.parents.push(msg);
	}
	
	
	pub fn pop_parent(&mut self)
	{
		self.parents.pop();
	}
	
	
	pub fn has_messages(&self) -> bool
	{
		self.messages.len() != 0
	}
	
	
	pub fn has_errors(&self) -> bool
	{
		self.messages.len() != 0
	}
	
	
	pub fn has_error_at(&self, fileserver: &FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		for msg in &self.messages
		{
			if self.msg_has_error_at(msg, fileserver, filename, line, error_excerpt)
				{ return true; }
		}
		
		false
	}
	
	
	fn msg_has_error_at(&self, msg: &Message, fileserver: &FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		match msg.inner
		{
			Some(ref inner) => match self.msg_has_error_at(&inner, fileserver, filename, line, error_excerpt)
			{
				true => return true,
				false => { }
			}
			
			None => { }
		}
	
		if !msg.descr.contains(error_excerpt)
			{ return false; }
			
		if msg.span.is_none()
			{ return false; }
			
		let span = msg.span.as_ref().unwrap();
		
		if &*span.file != filename
			{ return false; }
		
		if span.location.is_none()
			{ return false; }
			
		let location = span.location.unwrap();
		
		let chars = fileserver.get_chars(RcReport::new(), &*span.file, None).ok().unwrap();
		let counter = CharCounter::new(&chars);
		
		let (span_line, _) = counter.get_line_column_at_index(location.0);
		
		if span_line != line
			{ return false; }
			
		true
	}
	
	
	pub fn print_all(&self, fileserver: &FileServer)
	{
		enable_windows_ansi_support();
		
		for msg in &self.messages
		{
			self.print_msg(fileserver, msg, 0);
			println!();
		}
	}
	
	
	fn print_msg(&self, fileserver: &FileServer, msg: &Message, indent: usize)
	{
		match msg.span
		{
			None =>
			{
				// Print description without location information.
				print!("{}", C_ERROR);
				println!("error: {}", msg.descr);
				print!("{}", C_DEFAULT);
			}
			
			Some(ref span) =>
			{
				match span.location
				{
					None =>
					{
						// Print description with filename but without position information.
						self.print_indent(indent);
						print!("{}", C_LOCATION);
						println!("{}:", *span.file);
						print!("{}", C_ERROR);
						println!("error: {}", msg.descr);
						print!("{}", C_DEFAULT);
					}
					
					Some((start, end)) =>
					{
						// Print location information.
						let chars = fileserver.get_chars(RcReport::new(), &*span.file, None).ok().unwrap();
						let counter = CharCounter::new(&chars);
						
						let (line1, col1) = counter.get_line_column_at_index(start);
						let (line2, col2) = counter.get_line_column_at_index(end);
						
						self.print_indent(indent);
						print!("{}", C_LOCATION);
						println!("{}:{}:{} {}:{}:",
							*span.file,
							line1 + 1, col1 + 1,
							line2 + 1, col2 + 1);
							
						self.print_indent(indent);
						print!("{}", C_ERROR);
						println!("error: {}", msg.descr);
						print!("{}", C_DEFAULT);
						
						// Print annotated source code.
						self.print_msg_src(&counter, line1, col1, line2, col2, indent);
					}
				}
			}
		}
		
		match msg.inner
		{
			Some(ref inner) => self.print_msg(fileserver, &inner, indent + 1),
			None => { }
		}
	}
	
	
	fn print_indent(&self, indent: usize)
	{
		for _ in 0..indent
			{ print!("     "); }
	}
	
	
	fn print_msg_src(&self, counter: &CharCounter, line1: usize, col1: usize, line2: usize, col2: usize, indent: usize)
	{
		let first_line = if (line1 as isize - 2) < 0
			{ 0 }
		else
			{ line1 - 2 };
			
		
		let last_line = if (line2 + 3) >= counter.get_line_count()
			{ counter.get_line_count() }
		else
			{ line2 + 3 };
		
		
		// Print annotated source lines.
		for line in first_line..last_line
		{
			self.print_indent(indent);
			print!("{}", C_LINENUM);
			print!("{:4} | ", line + 1);
			print!("{}", C_SRC);
			
			let line_pos = counter.get_index_range_of_line(line);
			let excerpt = counter.get_excerpt(line_pos.0, line_pos.1);
			
			// Print source line excerpt.
			for p in 0..excerpt.len()
			{
				// Add a space for spans of zero characters.
				if line == line1 && line == line2 && p == col1 && p == col2
					{ print!(" "); }
				
				let c = excerpt[p];
				
				if c == '\t'
					{ print!("  "); }
				else if c <= ' '
					{ print!(" "); }
				else
					{ print!("{}", c); }
			}
			
			println!("");
			
			// Print markings on line below, if contained in span.
			if line >= line1 && line <= line2
			{
				self.print_indent(indent);
				print!("{}", C_LINENUM);
				print!("     | ");
				print!("{}", C_ERROR);
				
				for p in 0..excerpt.len()
				{
					// Print markings for spans of zero characters.
					if p == col1 && p == col2
						{ print!("^"); }
						
					let marking = if p >= col1 && p < col2
						{ "^" }
					else
						{ " " };
				
					if excerpt[p] == '\t'
						{ print!("{0}{0}", marking); }
					else
						{ print!("{}", marking); }
				}
				
				println!("");
			}
		}
		
		print!("{}", C_DEFAULT);
	}
}


impl RcReport
{
	pub fn new() -> RcReport
	{
		RcReport { report: Rc::new(RefCell::new(Report::new())) }
	}
	
	
	pub fn error<S>(&self, descr: S)
	where S: Into<String>
	{
		self.report.borrow_mut().error(descr);
	}
	
	
	pub fn error_span<S>(&self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.report.borrow_mut().error_span(descr, span);
	}
	
	
	pub fn push_parent<S>(&self, descr: S, span: &Span) -> ReportParentGuard
	where S: Into<String>
	{
		let guard = ReportParentGuard{ report: self.clone() };
		
		self.report.borrow_mut().push_parent(descr, span);
		
		guard
	}
	
	
	fn pop_parent(&self)
	{
		self.report.borrow_mut().pop_parent();
	}
	
	
	pub fn has_messages(&self) -> bool
	{
		self.report.borrow_mut().has_messages()
	}
	
	
	pub fn has_errors(&self) -> bool
	{
		self.report.borrow_mut().has_errors()
	}
	
	
	pub fn has_error_at(&self, fileserver: &FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		self.report.borrow_mut().has_error_at(fileserver, filename, line, error_excerpt)
	}
	
	
	pub fn print_all(&self, fileserver: &FileServer)
	{
		self.report.borrow_mut().print_all(fileserver);
	}
}


impl Drop for ReportParentGuard
{
	fn drop(&mut self)
	{
		self.report.pop_parent();
	}
}