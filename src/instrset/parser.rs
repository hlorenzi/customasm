use diagn::{Message, Reporter};
use syntax::{Token, TokenKind, Parser};
use syntax::excerpt_as_usize;
use ::InstrSet;
use instrset::Rule;


pub struct InstrSetParser<'a, 't>
{
	pub instrset: InstrSet,
	
	reporter: &'a mut Reporter,
	parser: Parser<'t>,
	
	align_was_set: bool
}


impl<'a, 't> InstrSetParser<'a, 't>
{
	pub fn new(reporter: &'a mut Reporter, tokens: &'t [Token]) -> InstrSetParser<'a, 't>
	{
		let instrset = InstrSet
		{
			align: 8,
			rules: Vec::new()
		};
		
		InstrSetParser
		{
			instrset: instrset,
			
			reporter: reporter,
			parser: Parser::new(tokens),
			
			align_was_set: false
		}
	}
	

	pub fn parse(mut self) -> Result<InstrSet, Message>
	{
		self.parse_directives()?;
		self.parse_rules()?;
		Ok(self.instrset)
	}
	

	fn parse_directives(&mut self) -> Result<(), Message>
	{
		while self.parser.maybe_expect(TokenKind::Hash).is_some()
		{
			let tk_name = self.parser.expect_msg(TokenKind::Identifier, "expected directive name")?;
			match tk_name.excerpt.as_ref().unwrap().as_ref()
			{
				"align" => self.parse_directive_align(&tk_name)?,
				
				_ => return Err(Message::error_span("unknown directive", &tk_name.span))
			}
			
			self.parser.expect_linebreak()?;
		}
	
		Ok(())
	}
	
	
	fn parse_directive_align(&mut self, tk_name: &Token) -> Result<(), Message>
	{
		let tk_align = self.parser.expect_msg(TokenKind::Number, "expected alignment value")?;
		
		if self.align_was_set
			{ return Err(Message::error_span("duplicate align directive", &tk_name.span)); }
			
		self.instrset.align = excerpt_as_usize(&tk_align.excerpt.unwrap(), &tk_align.span)?;
		self.align_was_set = true;
		
		Ok(())
	}
	

	fn parse_rules(&mut self) -> Result<(), Message>
	{
		while !self.parser.is_over()
		{
			self.parse_rule()?;
			self.parser.expect_linebreak()?;
		}
		
		Ok(())
	}
	

	fn parse_rule(&mut self) -> Result<(), Message>
	{
		let mut rule = Rule::new();
		
		self.parse_rule_pattern(&mut rule)?;
		self.parser.expect(TokenKind::Arrow)?;
		
		self.instrset.rules.push(rule);
		
		Ok(())
	}
	

	fn parse_rule_pattern(&mut self, rule: &mut Rule) -> Result<(), Message>
	{
		let mut prev_was_parameter = false;
		
		
		while !self.parser.next_is(0, TokenKind::Arrow)
		{
			let tk = self.parser.advance();
			
			// Force read an identifier at the start of the pattern.
			if rule.pattern_parts.len() == 0
			{
				if tk.kind != TokenKind::Identifier
					{ return Err(Message::error_span("expected identifier as first pattern token", &tk.span)); }
					
				rule.pattern_add_exact(&tk);
			}
			
			// Read a parameter.
			else if tk.kind == TokenKind::BraceOpen
			{
				// Check for consecutive parameters without a separating token.
				if prev_was_parameter
					{ return Err(Message::error_span("expected a separating token between parameters", &tk.span.before())); }
			
				self.parse_rule_parameter(rule)?;
				prev_was_parameter = true;
			}
			
			// Read an exact pattern part.
			else if tk.kind.is_allowed_pattern_token()
			{
				// Check for a stricter set of tokens if a parameter came just before.
				if prev_was_parameter && !tk.kind.is_allowed_after_pattern_parameter()
					{ return Err(Message::error_span("invalid pattern token after parameter", &tk.span)); }
				
				rule.pattern_add_exact(&tk);
				prev_was_parameter = false;
			}
			
			// Else, it's illegal to appear in a pattern.
			else
				{ return Err(Message::error_span("invalid pattern token", &tk.span)); }
		}
	
		Ok(())
	}
	

	fn parse_rule_parameter(&mut self, rule: &mut Rule) -> Result<(), Message>
	{
		let tk_name = self.parser.expect(TokenKind::Identifier)?;
		let cascadable = self.parser.maybe_expect(TokenKind::Exclamation).is_some();
		
		let name = tk_name.excerpt.unwrap().clone();
		
		if rule.param_exists(&name)
			{ return Err(Message::error_span("duplicate parameter name", &tk_name.span)); }
			
		rule.pattern_add_param(name, cascadable);
		
		self.parser.expect(TokenKind::BraceClose)?;
		
		Ok(())
	}
}