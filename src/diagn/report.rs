use crate::*;


#[derive(Clone)]
pub struct Report
{
	messages: Vec<Message>,
	parents: Vec<Message>,
	parent_cap: Vec<usize>,
}


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message
{
	pub descr: String,
	pub kind: MessageKind,
	pub span: Option<diagn::Span>,
	pub short_excerpt: bool,
	pub inner: Vec<Message>,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessageKind
{
	Error,
	Warning,
	Note,
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
		Message {
			descr: descr.into(),
			kind: MessageKind::Error,
			span: None,
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn error_span<S>(descr: S, span: diagn::Span) -> Message
	where S: Into<String>
	{
		Message {
			descr: descr.into(),
			kind: MessageKind::Error,
			span: Some(span),
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn warning<S>(descr: S) -> Message
	where S: Into<String>
	{
		Message {
			descr: descr.into(),
			kind: MessageKind::Warning,
			span: None,
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn warning_span<S>(descr: S, span: diagn::Span) -> Message
	where S: Into<String>
	{
		Message {
			descr: descr.into(),
			kind: MessageKind::Warning,
			span: Some(span),
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn note<S>(descr: S) -> Message
	where S: Into<String>
	{
		Message {
			descr: descr.into(),
			kind: MessageKind::Note,
			span: None,
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn note_span<S>(descr: S, span: diagn::Span) -> Message
	where S: Into<String>
	{
		Message {
			descr: descr.into(),
			kind: MessageKind::Note,
			span: Some(span),
			short_excerpt: false,
			inner: Vec::new(),
		}
	}
	
	
	pub fn short_note_span<S>(descr: S, span: diagn::Span) -> Message
	where S: Into<String>
	{
		Message {
			descr: descr.into(),
			kind: MessageKind::Note,
			span: Some(span),
			short_excerpt: true,
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


	pub fn fuse_topmost(msgs: Vec<Message>) -> Message
	{
		let mut topmost = Message {
			inner: Vec::new(),
			..msgs[0].clone()
		};

		for msg in msgs
		{
			topmost.inner.extend(msg.inner);
		}

		topmost
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
	
	
	fn apply_color(
		&self,
		styler: &mut util::StringStyler)
	{
		match self
		{
			&MessageKind::Error => styler.red(),
			&MessageKind::Warning => styler.yellow(),
			&MessageKind::Note => styler.cyan(),
		}
	}
}


impl Report
{
	pub fn new() -> Report
	{
		Report
		{
			messages: Vec::new(),
			parents: Vec::new(),
			parent_cap: Vec::new(),
		}
	}


	pub fn transfer_to(&mut self, other: &mut Report)
	{
		for msg in &self.messages
		{
			other.message(msg.clone());
		}
	}
	
	
	pub fn wrap_in_parents_capped(
		&self,
		mut msg: Message)
		-> Message
	{
		let cap_at = self.parent_cap.last().copied().unwrap_or(0);

		for parent in self.parents[cap_at..].iter().rev()
		{
			msg = Message {
				descr: parent.descr.clone(),
				kind: parent.kind,
				span: parent.span,
				short_excerpt: parent.short_excerpt,
				inner: vec![msg],
			};
		}
		
		msg
	}
	
	
	pub fn wrap_in_parents(
		&self,
		mut msg: Message)
		-> Message
	{
		for parent in self.parents.iter().rev()
		{
			msg = Message {
				descr: parent.descr.clone(),
				kind: parent.kind,
				span: parent.span,
				short_excerpt: parent.short_excerpt,
				inner: vec![msg],
			};
		}
		
		msg
	}
	
	
	pub fn wrap_in_parents_dedup(
		&self,
		mut msg: Message)
		-> Message
	{
		let mut seen_msgs = std::collections::HashSet::new();
		
		let mut filtered_parents = Vec::new();
		for parent in self.parents.iter()
		{
			let msg_key = format!(
				"{}_{:?}",
				parent.descr,
				parent.span);

			if seen_msgs.contains(&msg_key)
			{
				continue;
			}
			
			seen_msgs.insert(msg_key);

			filtered_parents.push(parent);
		}

		// Deduplicate "match attempted" messages from
		// a recursive asm block, leaving only the last one
		let mut filtered_parents2 = Vec::new();
		let mut seen_match_attempted_msgs = 0;
		for parent in filtered_parents.iter().rev()
		{
			if parent.descr.starts_with("match attempted")
			{
				if seen_match_attempted_msgs > 0
				{
					continue;
				}

				seen_match_attempted_msgs += 1;
			}
			else
			{
				seen_match_attempted_msgs = 0;
			}

			filtered_parents2.push(parent);
		}


		for parent in filtered_parents2.iter()
		{
			msg = Message {
				descr: parent.descr.clone(),
				kind: parent.kind,
				span: parent.span,
				short_excerpt: parent.short_excerpt,
				inner: vec![msg],
			};
		}
		
		msg
	}
	
	
	pub fn message(&mut self, msg: Message)
	{
		self.messages.push(self.wrap_in_parents(msg));
	}
	
	
	pub fn message_without_parents(&mut self, msg: Message)
	{
		self.messages.push(msg);
	}
	
	
	pub fn message_with_parents_dedup(
		&mut self,
		msg: Message)
	{
		self.messages.push(self.wrap_in_parents_dedup(msg));
	}
	
	
	pub fn push_multiple(&mut self, mut msgs: Vec<Message>)
	{
		let range = self.parent_cap.last().copied().unwrap_or(0);

		for parent in self.parents[range..].iter().rev()
		{
			let msg = Message {
				descr: parent.descr.clone(),
				kind: parent.kind,
				span: parent.span,
				short_excerpt: parent.short_excerpt,
				inner: msgs,
			};

			msgs = vec![msg];
		}
		
		for msg in msgs
		{
			self.messages.push(msg);
		}
	}


	pub fn stop_at_errors(&self) -> Result<(), ()>
	{
		for msg in &self.messages
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
	
	
	pub fn error_span<S>(&mut self, descr: S, span: diagn::Span)
	where S: Into<String>
	{
		self.message(Message::error_span(descr, span));
	}
	
	
	pub fn warning<S>(&mut self, descr: S)
	where S: Into<String>
	{
		self.message(Message::warning(descr));
	}
	
	
	pub fn warning_span<S>(&mut self, descr: S, span: diagn::Span)
	where S: Into<String>
	{
		self.message(Message::warning_span(descr, span));
	}
	
	
	pub fn note<S>(&mut self, descr: S)
	where S: Into<String>
	{
		self.message(Message::note(descr));
	}
	
	
	pub fn note_span<S>(&mut self, descr: S, span: diagn::Span)
	where S: Into<String>
	{
		self.message(Message::note_span(descr, span));
	}
	
	
	pub fn push_parent<S>(&mut self, descr: S, span: diagn::Span)
	where S: Into<String>
	{
		self.parents.push(Message::error_span(descr, span));
	}
	
	
	pub fn push_parent_note<S>(&mut self, descr: S, span: diagn::Span)
	where S: Into<String>
	{
		self.parents.push(Message::note_span(descr, span));
	}
	
	
	pub fn push_parent_short_note<S>(&mut self, descr: S, span: diagn::Span)
	where S: Into<String>
	{
		self.parents.push(Message::short_note_span(descr, span));
	}
	
	
	pub fn pop_parent(&mut self)
	{
		self.parents.pop().unwrap();
	}
	
	
	pub fn push_parent_cap(&mut self)
	{
		self.parent_cap.push(self.parents.len());
	}
	
	
	pub fn pop_parent_cap(&mut self)
	{
		self.parent_cap.pop().unwrap();
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
	
	
	pub fn has_message_at(&self, fileserver: &mut dyn util::FileServer, filename: &str, kind: MessageKind, line: usize, error_excerpt: &str) -> bool
	{
		for msg in &self.messages
		{
			if self.msg_has_error_at(msg, fileserver, filename, kind, line, error_excerpt)
				{ return true; }
		}
		
		false
	}
	
	
	pub fn has_error_at(&self, fileserver: &mut dyn util::FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		for msg in &self.messages
		{
			if self.msg_has_error_at(msg, fileserver, filename, MessageKind::Error, line, error_excerpt)
				{ return true; }
		}
		
		false
	}
	
	
	pub fn has_first_error_at(&self, fileserver: &mut dyn util::FileServer, filename: &str, line: usize, error_excerpt: &str) -> bool
	{
		if self.messages.len() == 0
			{ return false; }
			
		self.msg_has_error_at(&self.messages[0], fileserver, filename, MessageKind::Error, line, error_excerpt)
	}
	
	
	fn msg_has_error_at(
		&self,
		msg: &Message,
		fileserver: &mut dyn util::FileServer,
		filename: &str,
		kind: MessageKind,
		line: usize,
		error_excerpt: &str)
		-> bool
	{
		for inner in &msg.inner
		{
			if self.msg_has_error_at(
				&inner,
				fileserver,
				filename,
				kind,
				line,
				error_excerpt)
			{
				return true;
			}
		}

		if msg.kind != kind
			{ return false; }
	
		if !msg.descr.contains(error_excerpt)
			{ return false; }
			
		if msg.span.is_none()
			{ return true; }
			
		let file_handle =
			fileserver.get_handle(
				&mut diagn::Report::new(),
				None,
				filename)
			.unwrap();
		
		let span = msg.span.as_ref().unwrap();
		
		if span.file_handle != file_handle
			{ return false; }
		
		if span.location().is_none()
			{ return false; }
			
		let location = span.location().unwrap();
		
		let chars = fileserver.get_str_unwrap(file_handle);

		let counter = util::CharCounter::new(&chars);
		
		let (span_line, _) = counter.get_line_column_at_index(location.0);
		
		if span_line != line
			{ return false; }
			
		true
	}
	
	
	pub fn print_all(
		&self,
		writer: &mut dyn std::io::Write,
		fileserver: &dyn util::FileServer,
		use_colors: bool)
	{
		if self.messages.len() > 0
		{
			write!(writer, "").unwrap();
		}
		
		for msg in &self.messages
		{
			let mut styler = util::StringStyler::new(use_colors);

			self.print_msg(
				&mut styler,
				fileserver,
				msg,
				0);
				
			write!(writer, "{}", styler.result).unwrap();
		}
	}
	
	
	fn print_msg(
		&self,
		styler: &mut util::StringStyler,
		fileserver: &dyn util::FileServer,
		msg: &Message,
		indent: usize)
	{
		styler.indent(indent);

		styler.gray();
		if indent > 0
		{
			styler.add(" + ");
		}
		else
		{
			styler.bold();
		}

		msg.kind.apply_color(styler);
		styler.add(msg.kind.get_label());
		styler.add(": ");
		styler.addln(&msg.descr);

		styler.reset();

		let inner_indent = self.print_msg_src(
			styler,
			fileserver,
			msg,
			indent);
		
		for inner in &msg.inner
		{
			self.print_msg(
				styler,
				fileserver,
				&inner,
				indent + inner_indent);
			
			if indent == 0
			{
				styler.addln("");
			}
		}
	}


	fn get_line_info(
		&self,
		fileserver: &dyn util::FileServer,
		span: &diagn::Span,
		msg: &Message)
		-> LineInfo
	{
		let chars = fileserver.get_str_unwrap(span.file_handle);
			
		let counter = util::CharCounter::new(&chars);
		

		let (start, end) = span.location().unwrap();
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
		styler: &mut util::StringStyler,
		fileserver: &dyn util::FileServer,
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

		let filename = fileserver.get_filename(span.file_handle);

		if span.location().is_none()
		{
			// If no span, print only the filename.
			styler.indent(indent);
			styler.gray();
			styler.add(" --> ");
			styler.addln(filename);
			styler.reset();
			return 0;
		}

		let line_info = self.get_line_info(
			fileserver,
			span,
			msg);

		let chars = fileserver.get_str_unwrap(span.file_handle);
		let counter = util::CharCounter::new(&chars);

		// Print the filename and line/column information
		styler.indent(indent + line_info.label_width - 1);
		styler.gray();
		styler.add(" --> ");
		styler.add(filename);
		styler.add(":");
		styler.add(&format!("{}", line_info.line1 + 1));
		styler.add(":");
		styler.add(&format!("{}", line_info.col1 + 1));
		styler.addln(":");
		styler.reset();

		// Print annotated source lines
		for line in line_info.excerpt_line1..line_info.excerpt_line2
		{
			styler.indent(indent);
			styler.gray();
			styler.add(&format!("{:>1$}", line + 1, line_info.label_width));
			styler.add(" | ");

			styler.white();
			
			let line_pos = counter.get_index_range_of_line(line);
			let excerpt = counter
				.get_excerpt(line_pos.0, line_pos.1)
				.chars()
				.collect::<Vec<_>>();
			
			// Print an excerpt of the source line
			for p in 0..excerpt.len()
			{
				// Add a space for spans of zero characters
				if line == line_info.line1 &&
					line == line_info.line2 &&
					p == line_info.col1 &&
					p == line_info.col2
				{
					styler.add(" ");
				}
				
				match excerpt[p]
				{
					'\t' =>
						styler.add("  "),

					c @ _ if c <= ' ' =>
						styler.add(" "),

					c @ _ =>
						styler.add_char(c),
				}
			}
			
			styler.addln("");
			
			// Print markings on line below, if contained in span
			if line >= line_info.line1 &&
				line <= line_info.line2
			{
				styler.indent(indent);
				styler.gray();
				styler.add(&format!("{:>1$}", "", line_info.label_width));
				styler.add(" | ");
				msg.kind.apply_color(styler);
				
				for p in 0..(excerpt.len() + 1)
				{
					// Print marking for spans of zero characters
					if p == line_info.col1 &&
						p == line_info.col2 &&
						line_info.line1 == line_info.line2
					{
						styler.add("^");
					}
					
					// Print markings for all other spans
					let marking = {
						if (line > line_info.line1 || p >= line_info.col1) &&
							(line < line_info.line2 || p < line_info.col2)
							{ "^" }
						else
							{ " " }
					};
				
					if p < excerpt.len() && excerpt[p] == '\t'
					{
						// Double markings for tab characters
						styler.add(marking);
						styler.add(marking);
					}
					else
					{
						styler.add(marking);
					}
				}
				
				styler.addln("");
			}
		}
		
		styler.reset();
		
		// Return the new indentation for nested messages
		line_info.label_width
	}
}