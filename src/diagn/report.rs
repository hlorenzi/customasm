use diagn::Span;
use util::CharCounter;
use util::FileServer;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;


const C_DEFAULT:  &'static str = "\u{1B}[0m";
const C_LOCATION: &'static str = "\u{1B}[0m\u{1B}[90m";
const C_ERROR:    &'static str = "\u{1B}[0m\u{1B}[91m";
const C_WARNING:  &'static str = "\u{1B}[0m\u{1B}[93m";
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
	pub kind: MessageKind,
	pub span: Option<Span>,
	pub inner: Option<Box<Message>>
}


#[derive(Copy, Clone)]
enum MessageKind
{
	Error,
	Warning
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
			{ msg = Message{ descr: parent.descr.clone(), kind: parent.kind, span: parent.span.clone(), inner: Some(Box::new(msg)) }; }
		
		self.messages.push(msg);
	}
	
	
	pub fn error<S>(&mut self, descr: S)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), kind: MessageKind::Error, span: None, inner: None };
		self.message(msg);
	}
	
	
	pub fn error_span<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), kind: MessageKind::Error, span: Some(span.clone()), inner: None };
		self.message(msg);
	}
	
	
	pub fn warning<S>(&mut self, descr: S)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), kind: MessageKind::Warning, span: None, inner: None };
		self.message(msg);
	}
	
	
	pub fn warning_span<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), kind: MessageKind::Warning, span: Some(span.clone()), inner: None };
		self.message(msg);
	}
	
	
	pub fn push_parent<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		let msg = Message{ descr: descr.into(), kind: MessageKind::Error, span: Some(span.clone()), inner: None };
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
	
	
	pub fn print_all(&self, writer: &mut Write, fileserver: &FileServer)
	{
		for msg in &self.messages
		{
			self.print_msg(writer, fileserver, msg, 0);
			writeln!(writer).unwrap();
		}
	}
	
	
	fn print_msg(&self, writer: &mut Write, fileserver: &FileServer, msg: &Message, indent: usize)
	{
		let kind_label = msg.kind.get_label();
		let highlight_color = msg.kind.get_color();
		
		match msg.span
		{
			None =>
			{
				// Print description without location information.
				write!(writer, "{}", highlight_color).unwrap();
				writeln!(writer, "{}: {}", kind_label, msg.descr).unwrap();
				write!(writer, "{}", C_DEFAULT).unwrap();
			}
			
			Some(ref span) =>
			{
				match span.location
				{
					None =>
					{
						// Print description with filename but without position information.
						self.print_indent(writer, indent);
						write!(writer, "{}", C_LOCATION).unwrap();
						writeln!(writer, "{}:", *span.file).unwrap();
						write!(writer, "{}", highlight_color).unwrap();
						writeln!(writer, "{}: {}", kind_label, msg.descr).unwrap();
						write!(writer, "{}", C_DEFAULT).unwrap();
					}
					
					Some((start, end)) =>
					{
						// Print location information.
						let chars = fileserver.get_chars(RcReport::new(), &*span.file, None).ok().unwrap();
						let counter = CharCounter::new(&chars);
						
						let (line1, col1) = counter.get_line_column_at_index(start);
						let (line2, col2) = counter.get_line_column_at_index(end);
						
						self.print_indent(writer, indent);
						write!(writer, "{}", C_LOCATION).unwrap();
						writeln!(writer, "{}:{}:{} {}:{}:",
							*span.file,
							line1 + 1, col1 + 1,
							line2 + 1, col2 + 1).unwrap();
							
						self.print_indent(writer, indent);
						write!(writer, "{}", highlight_color).unwrap();
						writeln!(writer, "{}: {}", kind_label, msg.descr).unwrap();
						write!(writer, "{}", C_DEFAULT).unwrap();
						
						// Print annotated source code.
						self.print_msg_src(writer, &counter, msg.kind.get_color(), line1, col1, line2, col2, indent);
					}
				}
			}
		}
		
		match msg.inner
		{
			Some(ref inner) => self.print_msg(writer, fileserver, &inner, indent + 1),
			None => { }
		}
	}
	
	
	fn print_indent(&self, writer: &mut Write, indent: usize)
	{
		for _ in 0..indent
			{ write!(writer, "     ").unwrap(); }
	}
	
	
	fn print_msg_src(&self, writer: &mut Write, counter: &CharCounter, highlight_color: &'static str, line1: usize, col1: usize, line2: usize, col2: usize, indent: usize)
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
			self.print_indent(writer, indent);
			write!(writer, "{}", C_LINENUM).unwrap();
			write!(writer, "{:4} | ", line + 1).unwrap();
			write!(writer, "{}", C_SRC).unwrap();
			
			let line_pos = counter.get_index_range_of_line(line);
			let excerpt = counter.get_excerpt(line_pos.0, line_pos.1);
			
			// Print source line excerpt.
			for p in 0..excerpt.len()
			{
				// Add a space for spans of zero characters.
				if line == line1 && line == line2 && p == col1 && p == col2
					{ write!(writer, " ").unwrap(); }
				
				let c = excerpt[p];
				
				if c == '\t'
					{ write!(writer, "  ").unwrap(); }
				else if c <= ' '
					{ write!(writer, " ").unwrap(); }
				else
					{ write!(writer, "{}", c).unwrap(); }
			}
			
			writeln!(writer, "").unwrap();
			
			// Print markings on line below, if contained in span.
			if line >= line1 && line <= line2
			{
				self.print_indent(writer, indent);
				write!(writer, "{}", C_LINENUM).unwrap();
				write!(writer, "     | ").unwrap();
				write!(writer, "{}", highlight_color).unwrap();
				
				for p in 0..(excerpt.len() + 1)
				{
					// Print markings for spans of zero characters.
					if p == col1 && p == col2
						{ write!(writer, "^").unwrap(); }
						
					let marking = if p >= col1 && p < col2
						{ "^" }
					else
						{ " " };
				
					if p < excerpt.len() && excerpt[p] == '\t'
						{ write!(writer, "{0}{0}", marking).unwrap(); }
					else
						{ write!(writer, "{}", marking).unwrap(); }
				}
				
				writeln!(writer, "").unwrap();
			}
		}
		
		write!(writer, "{}", C_DEFAULT).unwrap();
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
	
	
	pub fn warning<S>(&self, descr: S)
	where S: Into<String>
	{
		self.report.borrow_mut().warning(descr);
	}
	
	
	pub fn warning_span<S>(&self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.report.borrow_mut().warning_span(descr, span);
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
	
	
	pub fn print_all(&self, writer: &mut Write, fileserver: &FileServer)
	{
		self.report.borrow_mut().print_all(writer, fileserver);
	}
}


impl Drop for ReportParentGuard
{
	fn drop(&mut self)
	{
		self.report.pop_parent();
	}
}


impl MessageKind
{
	fn get_label(&self) -> &'static str
	{
		match self
		{
			&MessageKind::Error => "error",
			&MessageKind::Warning => "warning"
		}
	}
	
	
	fn get_color(&self) -> &'static str
	{
		match self
		{
			&MessageKind::Error => C_ERROR,
			&MessageKind::Warning => C_WARNING
		}
	}
}