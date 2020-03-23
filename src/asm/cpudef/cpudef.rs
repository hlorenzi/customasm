use crate::start_flame;
use crate::syntax::{Token, TokenKind, Parser};
use crate::expr::{Expression, ExpressionValue};
use crate::asm::cpudef::{Rule, RuleParameterType, RulePatternMatcher};
use num_bigint::BigInt;
use std::collections::HashMap;


#[derive(Debug)]
pub struct CpuDef
{
	pub bits: usize,
	pub label_align: Option<usize>,
	pub rules: Vec<Rule>,
	pub pattern_matcher: RulePatternMatcher,
	pub custom_token_defs: Vec<CustomTokenDef>
}


struct CpuDefParser<'t>
{
	parser: &'t mut Parser,
	
	bits: Option<usize>,
	label_align: Option<usize>,
	rules: Vec<Rule>,
	custom_token_defs: Vec<CustomTokenDef>
}


#[derive(Debug)]
pub struct CustomTokenDef
{
	pub name: String,
	pub excerpt_to_value_map: HashMap<String, ExpressionValue>
}


impl CpuDef
{
	pub fn parse(parser: &mut Parser) -> Result<CpuDef, ()>
	{
		let report = parser.report.clone();
		
		let mut cpudef_parser = CpuDefParser
		{
			parser: parser,
			bits: None,
			label_align: None,
			rules: Vec::new(),
			custom_token_defs: Vec::new()
		};
		
		let mut _flame = start_flame("parse directives");
		cpudef_parser.parse_directives()?;	
		
		if cpudef_parser.bits.is_none()
			{ cpudef_parser.bits = Some(8); }

		_flame.drop_start("parse rules");
		cpudef_parser.parse_rules()?;
		
		_flame.drop_start("build pattern matcher");
		let pattern_matcher = RulePatternMatcher::new(report, &cpudef_parser.rules, &cpudef_parser.custom_token_defs)?;
		drop(_flame);
		
		//println!("[pattern tree for cpudef]");
		//pattern_matcher.print_debug();
		//println!();
		
		let cpudef = CpuDef
		{
			bits: cpudef_parser.bits.unwrap(),
			label_align: cpudef_parser.label_align,
			rules: cpudef_parser.rules,
			pattern_matcher: pattern_matcher,
			custom_token_defs: cpudef_parser.custom_token_defs
		};
		
		Ok(cpudef)
	}
}


impl<'t> CpuDefParser<'t>
{
	fn parse_directives(&mut self) -> Result<(), ()>
	{
		while self.parser.maybe_expect(TokenKind::Hash).is_some()
		{
			let tk_name = self.parser.expect_msg(TokenKind::Identifier, "expected directive name")?;
			match tk_name.excerpt.as_ref().unwrap().as_ref()
			{
				"align"      => self.parse_directive_align(&tk_name)?,
				"bits"       => self.parse_directive_bits(&tk_name)?,
				"labelalign" => self.parse_directive_labelalign(&tk_name)?,
				"tokendef"   => self.parse_directive_tokendef(&tk_name)?,
				
				_ => return Err(self.parser.report.error_span("unknown directive", &tk_name.span))
			}
			
			self.parser.expect_linebreak()?;
		}
	
		Ok(())
	}
	
	
	fn parse_directive_align(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		self.parser.report.warning_span("deprecated directive; use `#bits`", &tk_name.span);
		self.parse_directive_bits(tk_name)
	}
	
	
	fn parse_directive_bits(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let (tk_bits, bits) = self.parser.expect_usize()?;
		
		if self.bits.is_some()
			{ return Err(self.parser.report.error_span("duplicate `bits` directive", &tk_name.span)); }
			
		if bits == 0
			{ return Err(self.parser.report.error_span("invalid byte size", &tk_bits.span)); }
		
		self.bits = Some(bits);
		
		Ok(())
	}
	
	
	fn parse_directive_labelalign(&mut self, tk_name: &Token) -> Result<(), ()>
	{
		let (tk_value, value) = self.parser.expect_usize()?;
		
		if self.label_align.is_some()
			{ return Err(self.parser.report.error_span("duplicate `labelalign` directive", &tk_name.span)); }
			
		if value == 0
			{ return Err(self.parser.report.error_span("invalid alignment", &tk_value.span)); }
		
		self.label_align = Some(value);
		
		Ok(())
	}
	
	
	fn parse_directive_tokendef(&mut self, _tk_name: &Token) -> Result<(), ()>
	{
		let tk_defname = self.parser.expect(TokenKind::Identifier)?;
		
		let defname = tk_defname.excerpt.unwrap().clone();
		
		if self.custom_token_defs.iter().find(|def| def.name == defname).is_some()
			{ return Err(self.parser.report.error_span("duplicate tokendef name", &tk_defname.span)); }
		
		let mut tokendef = CustomTokenDef
		{
			name: defname,
			excerpt_to_value_map: HashMap::new()
		};
		
		self.parser.expect(TokenKind::BraceOpen)?;
		
		while !self.parser.is_over() && !self.parser.next_is(0, TokenKind::BraceClose)
		{
			let tk_token = self.parser.expect(TokenKind::Identifier)?;
			let token_excerpt = tk_token.excerpt.unwrap().clone();
			
			if tokendef.excerpt_to_value_map.contains_key(&token_excerpt)
				{ return Err(self.parser.report.error_span("duplicate tokendef entry", &tk_token.span)); }
			
			self.parser.expect(TokenKind::Equal)?;
			let value = ExpressionValue::Integer(BigInt::from(self.parser.expect_usize()?.1));
			
			tokendef.excerpt_to_value_map.insert(token_excerpt, value);
			
			if self.parser.maybe_expect_linebreak().is_some()
				{ continue; }
				
			if self.parser.next_is(0, TokenKind::BraceClose)
				{ continue; }
				
			self.parser.expect(TokenKind::Comma)?;
		}
		
		self.parser.expect(TokenKind::BraceClose)?;
		
		self.custom_token_defs.push(tokendef);
		
		Ok(())
	}
	

	fn parse_rules(&mut self) -> Result<(), ()>
	{
		while !self.parser.is_over() && !self.parser.next_is(0, TokenKind::BraceClose)
		{
			self.parse_rule()?;
			self.parser.expect_linebreak()?;
		}
		
		Ok(())
	}
	

	fn parse_rule(&mut self) -> Result<(), ()>
	{
		let mut rule = Rule::new();
		
		let pattern_span_start = self.parser.next().span.clone();
	
		self.parse_rule_pattern(&mut rule)?;
		
		if rule.pattern_parts.len() == 0
		{
			let span = self.parser.next().span.before();
			return Err(self.parser.report.error_span("empty rule pattern", &span));
		}
		
		rule.pattern_span = pattern_span_start.join(&self.parser.prev().span);
		
		self.parser.expect(TokenKind::Arrow)?;
		self.parse_rule_production(&mut rule)?;
		
		self.rules.push(rule);
		
		Ok(())
	}
	

	fn parse_rule_pattern(&mut self, rule: &mut Rule) -> Result<(), ()>
	{
		let mut prev_was_expr_parameter = false;
		
		while !self.parser.next_is(0, TokenKind::Arrow) && !self.parser.next_is(0, TokenKind::ColonColon)
		{
			let tk = self.parser.advance();
			
			let is_beginning_of_pattern = rule.pattern_parts.len() == 0;
			let mut is_allowed_at_beginning = false;
			
			// Read a parameter.
			if tk.kind == TokenKind::BraceOpen
			{
				// Check for consecutive parameters without a separating token.
				if prev_was_expr_parameter
					{ return Err(self.parser.report.error_span("expected a separating token between parameters; try using a `,`", &tk.span.before())); }
			
				prev_was_expr_parameter = self.parse_rule_parameter(rule)?;
				
				is_allowed_at_beginning = !prev_was_expr_parameter;
			}
			
			// Read an exact pattern part.
			else if tk.kind.is_allowed_pattern_token()
			{
				// Check for a stricter set of tokens if an expression parameter came just before.
				if prev_was_expr_parameter && !tk.kind.is_allowed_after_pattern_parameter()
					{ return Err(self.parser.report.error_span("potentially ambiguous token after parameter; try using a separating `,`", &tk.span)); }
					
				if tk.kind == TokenKind::Identifier
					{ is_allowed_at_beginning = true; }
				
				rule.pattern_add_exact(&tk);
				prev_was_expr_parameter = false;
			}
			
			// Check for end of file.
			else if tk.kind == TokenKind::End
				{ return Ok(()) }
			
			// Else, it's illegal to appear in a pattern.
			else
				{ return Err(self.parser.report.error_span("invalid pattern token", &tk.span)); }
				
			// Error if the pattern didn't start with an identifier or tokendef parameter.
			if is_beginning_of_pattern && !is_allowed_at_beginning
				{ return Err(self.parser.report.error_span("expected identifier as first pattern token", &tk.span.before())); }
		}
	
		Ok(())
	}
	

	fn parse_rule_parameter(&mut self, rule: &mut Rule) -> Result<bool, ()>
	{
		let tk_name = self.parser.expect(TokenKind::Identifier)?;
		
		let name = tk_name.excerpt.unwrap().clone();
		
		if is_reserved_var(&name)
			{ return Err(self.parser.report.error_span("reserved variable name", &tk_name.span)); }
		
		if rule.param_exists(&name)
			{ return Err(self.parser.report.error_span("duplicate parameter name", &tk_name.span)); }
			
		let (is_expr_parameter, typ) =
			if self.parser.maybe_expect(TokenKind::Colon).is_some()
				{ (false, self.parse_rule_parameter_type()?) }
			else
				{ (true, RuleParameterType::Expression) };
			
		rule.pattern_add_param(name, typ);
		
		self.parser.expect(TokenKind::BraceClose)?;
		
		Ok(is_expr_parameter)
	}


	fn parse_rule_parameter_type(&mut self) -> Result<RuleParameterType, ()>
	{
		let tk_type = self.parser.expect(TokenKind::Identifier)?;
		let typename = tk_type.excerpt.unwrap().clone();
		let typename_first_char = typename.chars().next();

		if typename_first_char == Some('u') ||
			typename_first_char == Some('s') ||
			typename_first_char == Some('i')
		{
			if let Ok(type_width) = usize::from_str_radix(&typename[1..], 10)
			{
				match typename_first_char
				{
					Some('u') => return Ok(RuleParameterType::Unsigned(type_width)),
					Some('s') => return Ok(RuleParameterType::Signed(type_width)),
					Some('i') => return Ok(RuleParameterType::Integer(type_width)),
					_ => unreachable!()
				}
			}
		}
		
		let mut tokendef_index = None;
		for i in 0..self.custom_token_defs.len()
		{
			if typename == self.custom_token_defs[i].name
				{ tokendef_index = Some(i); }
		}

		if let Some(tokendef_index) = tokendef_index
			{ Ok(RuleParameterType::CustomTokenDef(tokendef_index)) }
		else
			{ Err(self.parser.report.error_span("unknown parameter type", &tk_type.span)) }
	}
	

	fn parse_rule_production(&mut self, rule: &mut Rule) -> Result<(), ()>
	{
		let expr = Expression::parse_for_rule(&mut self.parser, &rule.params)?;
		
		let width = match expr.width()
		{
			Some(w) => w,
			None => return Err(self.parser.report.error_span("width of expression not known; try using a bit slice like `x[hi:lo]`", &expr.returned_value_span()))
		};
		
		if width % self.bits.unwrap() != 0
			{ return Err(self.parser.report.error_span(format!("expression width (= {}) is not a multiple of the CPU's byte width (= {})", width, self.bits.unwrap()), &expr.returned_value_span())); }
		
		rule.production = expr;
		
		Ok(())
	}
}


fn is_reserved_var(name: &str) -> bool
{
	name == "pc"
}