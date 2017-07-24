use diagn::{Span, Message, Reporter};
use syntax::{Token, TokenKind, Parser};
use syntax::{excerpt_as_string_contents, excerpt_as_usize};
use expr::{ExpressionType, ExpressionParser};
use instrset::{InstrSet, Rule};


pub struct InstrSetParser<'a, 't>
{
	pub instrset: InstrSet,
	
	_reporter: &'a mut Reporter,
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
			
			_reporter: reporter,
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
		
		while self.parser.maybe_expect(TokenKind::ColonColon).is_some()
			{ self.parse_rule_constraint(&mut rule)?; }
		
		self.parser.expect(TokenKind::Arrow)?;
		self.parse_rule_production(&mut rule)?;
		
		self.instrset.rules.push(rule);
		
		Ok(())
	}
	

	fn parse_rule_pattern(&mut self, rule: &mut Rule) -> Result<(), Message>
	{
		let mut prev_was_parameter = false;
		
		
		while !self.parser.next_is(0, TokenKind::Arrow) && !self.parser.next_is(0, TokenKind::ColonColon)
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
		
		let name = tk_name.excerpt.unwrap().clone();
		
		if is_reserved_var(&name)
			{ return Err(Message::error_span("reserved variable name", &tk_name.span)); }
		
		if rule.param_exists(&name)
			{ return Err(Message::error_span("duplicate parameter name", &tk_name.span)); }
			
		rule.pattern_add_param(name);
		
		self.parser.expect(TokenKind::BraceClose)?;
		
		Ok(())
	}
	

	fn parse_rule_constraint(&mut self, rule: &mut Rule) -> Result<(), Message>
	{
		let expr = ExpressionParser::new(&mut self.parser).parse()?;
		
		expr.check_vars(&|name, span| expr_check_var(rule, name, span))?;
		
		if expr.eval_type(&|name| expr_get_var_type(rule, name))? != ExpressionType::Bool
			{ return Err(Message::error_span("expected bool expression for constraint", &expr.span())) }
			
		let descr = if self.parser.maybe_expect(TokenKind::Comma).is_some()
		{
			let tk_descr = self.parser.expect(TokenKind::String)?;
			Some(excerpt_as_string_contents(&tk_descr.excerpt.unwrap(), &tk_descr.span)?)
		}
		else
			{ None };
		
		rule.constraint_add(expr, descr);
		Ok(())
	}
	

	fn parse_rule_production(&mut self, rule: &mut Rule) -> Result<(), Message>
	{
		let mut total_span = self.parser.next().span;
		let mut total_width = 0;
		
		loop
		{
			let expr = ExpressionParser::new(&mut self.parser).parse()?;
			
			expr.check_vars(&|name, span| expr_check_var(rule, name, span))?;
			
			if expr.eval_type(&|name| expr_get_var_type(rule, name))? != ExpressionType::Integer
				{ return Err(Message::error_span("expected integer expression for production", &expr.span())) }
				
			match expr.width()
			{
				Some(w) => total_width += w,
				None => return Err(Message::error_span("width of expression not known; use a bit slice", &expr.span()))
			}
			
			total_span = total_span.join(&expr.span());
			
			rule.production_parts.push(expr);
			
			if self.parser.maybe_expect(TokenKind::Comma).is_none()
				{ break; }
		}
		
		if total_width % self.instrset.align != 0
			{ return Err(Message::error_span(format!("production (width = {}) does not align with a word boundary", total_width), &total_span)); }
		
		Ok(())
	}
}


fn is_reserved_var(name: &str) -> bool
{
	name == "pc"
}

	
fn expr_check_var(rule: &Rule, name: &str, span: &Span) -> Result<(), Message>
{
	if rule.param_exists(name) || is_reserved_var(name)
		{ Ok(()) }
	else
		{ Err(Message::error_span("unknown variable", span)) }
}
	
	
fn expr_get_var_type(_rule: &Rule, _name: &str) -> ExpressionType
{
	// All variables are integer type for now.
	ExpressionType::Integer
}