use crate::diagn::Span;
use crate::util::CharCounter;
use crate::util::FileServer;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;


const C_DEFAULT:  &'static str = "\u{001b}[0m";
const C_LOCATION: &'static str = "\u{001b}[90m";
const C_ERROR:    &'static str = "\u{001b}[91m";
const C_WARNING:  &'static str = "\u{001b}[93m";
const C_NOTE:     &'static str = "\u{001b}[96m";
const C_LINENUM:  &'static str = "\u{001b}[90m";
const C_SRC:      &'static str = "\u{001b}[97m";
const C_BOLD:     &'static str = "\u{001b}[1m";


pub struct Report
{
	messages: Vec<Message>,
	parents: Vec<Message>
}


#[derive(Clone)]
pub struct Message
{
	pub descr: String,
	pub kind: MessageKind,
	pub span: Option<Span>,
	pub short_excerpt: bool,
	pub inner: Vec<Message>,
}


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MessageKind
{
	Error,
	Warning,
	Note,
}


pub struct ReportErrorGuard(usize);


#[derive(Clone)]
pub struct RcReport
{
	report: Rc<RefCell<Report>>
}


pub struct ReportParentGuard
{
	report: RcReport
}


struct LineInfo
{
	line1: usize,
	col1: usize,
	line2: usize,
	col2: usize,
	excerpt_line1: usize,
	excerpt_line2: usize,
	label_width: usize,
}


impl Message
{
	pub fn error<S>(descr: S) -> Message
	where S: Into<String>
	{
		Message
		{
			descr: descr.into(),
			kind: MessageKind::Error,
			span: None,
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn error_span<S>(descr: S, span: &Span) -> Message
	where S: Into<String>
	{
		Message
		{
			descr: descr.into(),
			kind: MessageKind::Error,
			span: Some(span.clone()),
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn warning<S>(descr: S) -> Message
	where S: Into<String>
	{
		Message
		{
			descr: descr.into(),
			kind: MessageKind::Warning,
			span: None,
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn warning_span<S>(descr: S, span: &Span) -> Message
	where S: Into<String>
	{
		Message
		{
			descr: descr.into(),
			kind: MessageKind::Warning,
			span: Some(span.clone()),
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn note<S>(descr: S) -> Message
	where S: Into<String>
	{
		Message
		{
			descr: descr.into(),
			kind: MessageKind::Note,
			span: None,
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn note_span<S>(descr: S, span: &Span) -> Message
	where S: Into<String>
	{
		Message
		{
			descr: descr.into(),
			kind: MessageKind::Note,
			span: Some(span.clone()),
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn len_with_inner(&self) -> usize
	{
		let mut count = 1;
		for msg in &self.inner
		{
			count += msg.len_with_inner();
		}

		count
	}
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


	fn transfer(&mut self, other: &mut Report)
	{
		for msg in &self.messages
		{
			other.message(msg.clone());
		}
	}
	
	
	fn message(&mut self, mut msg: Message)
	{
		for parent in self.parents.iter().rev()
		{
			msg = Message {
				descr: parent.descr.clone(),
				kind: parent.kind,
				span: parent.span.clone(),
				short_excerpt: false,
				inner: vec![msg],
			};
		}
		
		self.messages.push(msg);
	}
	
	
	pub fn push_multiple(&mut self, mut msgs: Vec<Message>)
	{
		for parent in self.parents.iter().rev()
		{
			let msg = Message {
				descr: parent.descr.clone(),
				kind: parent.kind,
				span: parent.span.clone(),
				short_excerpt: false,
				inner: msgs,
			};

			msgs = vec![msg];
		}
		
		for msg in msgs
		{
			self.messages.push(msg);
		}
	}


	pub fn get_error_guard(&self) -> ReportErrorGuard
	{
		ReportErrorGuard(self.messages.len())
	}


	pub fn stop_at_errors(&self, since: ReportErrorGuard) -> Result<(), ()>
	{
		for msg in &self.messages[since.0..]
		{
			if let MessageKind::Error = msg.kind
			{
				return Err(());
			}
		}

		Ok(())
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
	
	
	pub fn warning<S>(&mut self, descr: S)
	where S: Into<String>
	{
		self.message(Message::warning(descr));
	}
	
	
	pub fn warning_span<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.message(Message::warning_span(descr, span));
	}
	
	
	pub fn note<S>(&mut self, descr: S)
	where S: Into<String>
	{
		self.message(Message::note(descr));
	}
	
	
	pub fn note_span<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.message(Message::note_span(descr, span));
	}
	
	
	pub fn push_parent<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.parents.push(Message::error_span(descr, span));
	}
	
	
	pub fn push_parent_note<S>(&mut self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.parents.push(Message::note_span(descr, span));
	}
	
	
	pub fn pop_parent(&mut self)
	{
		self.parents.pop();
	}
	
	
	pub fn len(&self) -> usize
	{
		self.messages.len()
	}
	
	
	pub fn len_with_inner(&self) -> usize
	{
		let mut count = 0;
		for msg in &self.messages
		{
			count += msg.len_with_inner();
		}

		count
	}
	
	
	pub fn has_messages(&self) -> bool
	{
		self.messages.len() != 0
	}
	
	
	pub fn has_errors(&self) -> bool
	{
		self.messages.len() != 0
	}
	
	
	pub fn has_message_at(&self, fileserver: &dyn FileServer, filename: &str, kind: MessageKind, line: usize, error_excerpt: &str) -> bool
	{
		for msg in &self.messages
		{
			if self.msg_has_error_at(msg, fileserver, filename, kind, line, error_excerpt)
				{ return true; }
		}
		
		false
	}
	
	
	pub fn has_error_at(&self, fileserver: &dyn FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		for msg in &self.messages
		{
			if self.msg_has_error_at(msg, fileserver, filename, MessageKind::Error, line, error_excerpt)
				{ return true; }
		}
		
		false
	}
	
	
	pub fn has_first_error_at(&self, fileserver: &dyn FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		if self.messages.len() == 0
			{ return false; }
			
		self.msg_has_error_at(&self.messages[0], fileserver, filename, MessageKind::Error, line, error_excerpt)
	}
	
	
	fn msg_has_error_at(&self, msg: &Message, fileserver: &dyn FileServer, filename: &str, kind: MessageKind, line: usize, error_excerpt: &str) -> bool
	{
		for inner in &msg.inner
		{
			if self.msg_has_error_at(&inner, fileserver, filename, kind, line, error_excerpt)
			{
				return true;
			}
		}

		if msg.kind != kind
			{ return false; }
	
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
	
	
	pub fn print_all(&self, writer: &mut dyn Write, fileserver: &dyn FileServer)
	{
		for msg in &self.messages
		{
			self.print_msg(writer, fileserver, msg, 0);
			writeln!(writer).unwrap();
		}
	}
	
	
	fn print_msg(&self, writer: &mut dyn Write, fileserver: &dyn FileServer, msg: &Message, indent: usize)
	{
		let kind_label = msg.kind.get_label();
		let highlight_color = msg.kind.get_color();
		
		self.print_indent(writer, indent);
		write!(writer, "{}", C_LOCATION).unwrap();
		if indent > 0
		{
			write!(writer, "[!] ").unwrap();
		}
		else
		{
			write!(writer, "{}", C_BOLD).unwrap();
		}

		write!(writer, "{}", highlight_color).unwrap();
		writeln!(writer, "{}: {}", kind_label, msg.descr).unwrap();
		write!(writer, "{}", C_DEFAULT).unwrap();

		let inner_indent = self.print_msg_src(
			writer,
			fileserver,
			msg,
			indent);
		
		for inner in &msg.inner
		{
			self.print_msg(writer, fileserver, &inner, indent + inner_indent);
			if indent == 0
			{
				writeln!(writer).unwrap();
			}
		}
	}
	
	
	fn print_indent(&self, writer: &mut dyn Write, indent: usize)
	{
		for _ in 0..indent
			{ write!(writer, " ").unwrap(); }
	}


	fn get_line_info(
		&self,
		fileserver: &dyn FileServer,
		span: &Span,
		msg: &Message)
		-> LineInfo
	{
		let chars = fileserver.get_chars(
				RcReport::new(),
				&span.file,
				None)
			.ok()
			.unwrap();
			
		let counter = CharCounter::new(&chars);
		

		let (start, end) = span.location.unwrap();
		let (line1, col1) = counter.get_line_column_at_index(start);
		let (line2, col2) = counter.get_line_column_at_index(end);


		let lines_before = {
			if msg.short_excerpt
				{ 0 }
			else
				{ 2 }
		};

		let lines_after = {
			if msg.short_excerpt
				{ 1 }
			else
				{ 3 }
		};
		

		let excerpt_line1 = if line1 < lines_before
			{ 0 }
		else
			{ line1 - lines_before };
			
		
		let excerpt_line2 = if (line2 + lines_after) >= counter.get_line_count()
			{ counter.get_line_count() }
		else
			{ line2 + lines_after };
		
		
		let line_label_width =
			format!(
				"{}",
				std::cmp::max(excerpt_line1, excerpt_line2))
			.len();

		LineInfo {
			line1,
			col1,
			line2,
			col2,
			label_width: line_label_width,
			excerpt_line1,
			excerpt_line2,
		}
	}
	
	
	fn print_msg_src(
		&self,
		writer: &mut dyn Write,
		fileserver: &dyn FileServer,
		msg: &Message,
		indent: usize)
		-> usize
	{
		let span = {
			match msg.span
			{
				Some(ref span) => span,
				None => return 0,
			}
		};

		let highlight_color = msg.kind.get_color();

		// Print filename.
		if span.location.is_none()
		{
			self.print_indent(writer, indent);
			write!(writer, "{} --> ", C_LOCATION).unwrap();
			write!(writer, "{}:", span.file).unwrap();
			write!(writer, "{}", C_DEFAULT).unwrap();
			writeln!(writer).unwrap();
			return 0;
		}

		let line_info = self.get_line_info(
			fileserver,
			span,
			msg);

		self.print_indent(writer, indent + line_info.label_width - 1);
		write!(writer, "{} --> ", C_LOCATION).unwrap();
		write!(writer, "{}:", span.file).unwrap();
		write!(writer, "{}", C_DEFAULT).unwrap();

		// Print location information.
		let chars = fileserver.get_chars(RcReport::new(), &span.file, None).ok().unwrap();
		let counter = CharCounter::new(&chars);
		
		write!(writer, "{}", C_LOCATION).unwrap();
		writeln!(writer, "{}:{}:", line_info.line1 + 1, line_info.col1 + 1).unwrap();


		// Print annotated source lines.
		for line in line_info.excerpt_line1..line_info.excerpt_line2
		{
			self.print_indent(writer, indent);
			write!(writer, "{}", C_LINENUM).unwrap();
			write!(writer, "{:>1$} | ", line + 1, line_info.label_width).unwrap();
			write!(writer, "{}", C_SRC).unwrap();
			
			let line_pos = counter.get_index_range_of_line(line);
			let excerpt = counter.get_excerpt(line_pos.0, line_pos.1);
			
			// Print source line excerpt.
			for p in 0..excerpt.len()
			{
				// Add a space for spans of zero characters.
				if line == line_info.line1 &&
					line == line_info.line2 &&
					p == line_info.col1 &&
					p == line_info.col2
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
			if line >= line_info.line1 && line <= line_info.line2
			{
				self.print_indent(writer, indent);
				write!(writer, "{}", C_LINENUM).unwrap();
				write!(writer, "{:>1$} | ", "", line_info.label_width).unwrap();
				write!(writer, "{}", highlight_color).unwrap();
				
				for p in 0..(excerpt.len() + 1)
				{
					// Print markings for spans of zero characters.
					if p == line_info.col1 &&
						p == line_info.col2 &&
						line_info.line1 == line_info.line2
						{ write!(writer, "^").unwrap(); }
						
					let marking = if (line > line_info.line1 || p >= line_info.col1) && (line < line_info.line2 || p < line_info.col2)
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
		
		line_info.label_width
	}
}


impl RcReport
{
	pub fn new() -> RcReport
	{
		RcReport { report: Rc::new(RefCell::new(Report::new())) }
	}


	pub fn transfer_to(&self, other: RcReport)
	{
		self.report.borrow_mut().transfer(&mut other.report.borrow_mut());
	}


	pub fn into_inner(self) -> Report
	{
		Rc::try_unwrap(self.report).ok().unwrap().into_inner()
	}


	pub fn push_multiple(&self, msgs: Vec<Message>)
	{
		self.report.borrow_mut().push_multiple(msgs);
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
	
	
	pub fn note<S>(&self, descr: S)
	where S: Into<String>
	{
		self.report.borrow_mut().note(descr);
	}
	
	
	pub fn note_span<S>(&self, descr: S, span: &Span)
	where S: Into<String>
	{
		self.report.borrow_mut().note_span(descr, span);
	}
	
	
	pub fn push_parent<S>(&self, descr: S, span: &Span) -> ReportParentGuard
	where S: Into<String>
	{
		let guard = ReportParentGuard{ report: self.clone() };
		
		self.report.borrow_mut().push_parent(descr, span);
		
		guard
	}
	
	pub fn push_parent_note<S>(&self, descr: S, span: &Span) -> ReportParentGuard
	where S: Into<String>
	{
		let guard = ReportParentGuard{ report: self.clone() };
		
		self.report.borrow_mut().push_parent_note(descr, span);
		
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
	
	
	pub fn len(&self) -> usize
	{
		self.report.borrow().len()
	}
	
	
	pub fn len_with_inner(&self) -> usize
	{
		self.report.borrow().len_with_inner()
	}
	
	
	pub fn has_message_at(&self, fileserver: &dyn FileServer, filename: &str, kind: MessageKind, line: usize, error_excerpt: &str) -> bool
	{
		self.report.borrow_mut().has_message_at(fileserver, filename, kind, line, error_excerpt)
	}
	
	
	pub fn has_error_at(&self, fileserver: &dyn FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		self.report.borrow_mut().has_error_at(fileserver, filename, line, error_excerpt)
	}
	
	
	pub fn has_first_error_at(&self, fileserver: &dyn FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		self.report.borrow_mut().has_first_error_at(fileserver, filename, line, error_excerpt)
	}
	
	
	pub fn print_all(&self, writer: &mut dyn Write, fileserver: &dyn FileServer)
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
			&MessageKind::Warning => "warning",
			&MessageKind::Note => "note",
		}
	}
	
	
	fn get_color(&self) -> &'static str
	{
		match self
		{
			&MessageKind::Error => C_ERROR,
			&MessageKind::Warning => C_WARNING,
			&MessageKind::Note => C_NOTE,
		}
	}
}