use diagn::Span;
use syntax::{Token, TokenKind, Parser};


pub struct BankDef
{
	pub name: String,
	pub addr: usize,
	pub size: usize,
	pub outp: Option<usize>,
	pub fill: bool,
	pub decl_span: Option<Span>
}


struct BankDefParser<'p>
{
	parser: &'p mut Parser,
	span: Span,
	
	addr: Option<usize>,
	size: Option<usize>,
	outp: Option<usize>,
	fill: Option<bool>
}


impl BankDef
{
	pub fn new<S>(name: S, addr: usize, size: usize, outp: Option<usize>, fill: bool, decl_span: Option<Span>) -> BankDef
	where S: Into<String>
	{
		BankDef
		{
			name: name.into(),
			addr: addr,
			size: size,
			outp: outp,
			fill: fill,
			decl_span: decl_span
		}
	}
	
	
	pub fn parse<S>(name: S, parser: &mut Parser, span: &Span) -> Result<BankDef, ()>
	where S: Into<String>
	{
		let mut bankdef_parser = BankDefParser
		{
			parser: parser,
			span: span.clone(),
			
			addr: None,
			size: None,
			outp: None,
			fill: None
		};
		
		bankdef_parser.parse()?;
		
		let bankdef = BankDef
		{
			name: name.into(),
			addr: bankdef_parser.addr.unwrap(),
			size: bankdef_parser.size.unwrap(),
			outp: bankdef_parser.outp,
			fill: bankdef_parser.fill.unwrap_or(false),
			decl_span: Some(span.clone())
		};
		
		Ok(bankdef)
	}
}


impl<'p> BankDefParser<'p>
{
	pub fn parse(&mut self) -> Result<(), ()>
	{
		while !self.parser.is_over() && !self.parser.next_is(0, TokenKind::BraceClose)
		{
			self.parse_attribute()?;
			
			if self.parser.maybe_expect_linebreak().is_some()
				{ continue; }
				
			if self.parser.next_is(0, TokenKind::BraceClose)
				{ continue; }
				
			self.parser.expect(TokenKind::Comma)?;
		}
		
		if self.addr.is_none()
			{ return Err(self.parser.report.error_span("missing #addr attribute", &self.span)); }
		
		if self.size.is_none()
			{ return Err(self.parser.report.error_span("missing #size attribute", &self.span)); }
		
		Ok(())
	}
	
	
	pub fn parse_attribute(&mut self) -> Result<(), ()>
	{
		self.parser.expect(TokenKind::Hash)?;
		
		let tk_attrb_name = self.parser.expect(TokenKind::Identifier)?;
		let attrb_name = tk_attrb_name.excerpt.clone().unwrap();
		
		match attrb_name.as_ref()
		{
			"addr" => self.parse_attribute_addr(&tk_attrb_name),
			"size" => self.parse_attribute_size(&tk_attrb_name),
			"outp" => self.parse_attribute_outp(&tk_attrb_name),
			"fill" => self.parse_attribute_fill(&tk_attrb_name),
			_ => Err(self.parser.report.error_span("unknown attribute", &tk_attrb_name.span))
		}
	}
	
	
	pub fn parse_attribute_addr(&mut self, tk_attrb_name: &Token) -> Result<(), ()>
	{
		if self.addr.is_some()
			{ return Err(self.parser.report.error_span("duplicate #addr attribute", &tk_attrb_name.span)); }
			
		let (_, addr) = self.parser.expect_usize()?;
		
		self.addr = Some(addr);
		Ok(())
	}
	
	
	pub fn parse_attribute_size(&mut self, tk_attrb_name: &Token) -> Result<(), ()>
	{
		if self.size.is_some()
			{ return Err(self.parser.report.error_span("duplicate #size attribute", &tk_attrb_name.span)); }
			
		let (_, size) = self.parser.expect_usize()?;
		
		self.size = Some(size);
		Ok(())
	}
	
	
	pub fn parse_attribute_outp(&mut self, tk_attrb_name: &Token) -> Result<(), ()>
	{
		if self.outp.is_some()
			{ return Err(self.parser.report.error_span("duplicate #outp attribute", &tk_attrb_name.span)); }
			
		let (_, outp) = self.parser.expect_usize()?;
		
		self.outp = Some(outp);
		Ok(())
	}
	
	
	pub fn parse_attribute_fill(&mut self, tk_attrb_name: &Token) -> Result<(), ()>
	{
		if self.fill.is_some()
			{ return Err(self.parser.report.error_span("duplicate #fill attribute", &tk_attrb_name.span)); }
			
		self.fill = Some(true);
		Ok(())
	}
}