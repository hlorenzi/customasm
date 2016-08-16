use util::error::Error;
use util::expression::{Expression, ExpressionVariable};
use util::filehandler::{FileHandler, CustomFileHandler};
use util::parser::Parser;
use util::tokenizer;
use rule::{Rule, PatternSegment};
use std::path::Path;


pub struct Definition
{
	pub align_bits: usize,
	pub rules: Vec<Rule>
}


impl Definition
{
	pub fn from_file(filehandler: &FileHandler, filename: &Path) -> Result<Definition, Error>
	{
		let mut def = Definition
		{
			align_bits: 8,
			rules: Vec::new()
		};
		
		let chars = try!(filehandler.read_chars(filename));
		
		let tokens = tokenizer::tokenize(filename.to_string_lossy().into_owned(), &chars);
		let mut parser = Parser::new(&tokens);
		
		try!(def.parse_directives(&mut parser));
		try!(def.parse_rules(&mut parser));
		
		Ok(def)
	}
	
	
	pub fn from_str(s: &str) -> Result<Definition, Error>
	{
		let mut filehandler = CustomFileHandler::new();
		filehandler.add("<definition string>", s);
		
		Definition::from_file(&filehandler, &Path::new("<definition string>"))
	}


	fn parse_directives(&mut self, parser: &mut Parser) -> Result<(), Error>
	{
		while parser.match_operator(".")
		{
			let (directive, directive_span) = try!(parser.expect_identifier());
			
			match directive.as_ref()
			{
				"align" => self.align_bits = try!(parser.expect_number()).0,
				
				_ => return Err(Error::new_with_span(format!("unknown directive `{}`", directive), directive_span))
			}
			
			try!(parser.expect_linebreak_or_end());
		}
		
		Ok(())
	}


	fn parse_rules(&mut self, parser: &mut Parser) -> Result<(), Error>
	{
		while !parser.is_over()
		{
			let mut rule = Rule::new();
			
			try!(self.parse_pattern(parser, &mut rule));
			try!(parser.expect_operator("->"));
			try!(self.parse_production(parser, &mut rule));
			
			self.rules.push(rule);
			
			try!(parser.expect_linebreak_or_end());
		}
		
		Ok(())
	}


	fn parse_pattern(&self, parser: &mut Parser, rule: &mut Rule) -> Result<(), Error>
	{
		loop
		{
			if parser.current().is_identifier()
			{
				let (ident, _) = try!(parser.expect_identifier());
				rule.pattern_segments.push(PatternSegment::Exact(ident));
			}
			
			else if parser.match_operator("{")
			{
				let (name, name_span) = try!(parser.expect_identifier());
				
				if name == "pc" || name == "_"
					{ return Err(Error::new_with_span("reserved parameter name", name_span)); }
					
				if rule.check_parameter_exists(&name)
					{ return Err(Error::new_with_span(format!("duplicate parameter `{}`", name), name_span)); }
					
				let allow_unresolved = !parser.match_operator("!");
				
				let constraint =
					if parser.match_operator(":")
					{
						Some(try!(Expression::new_by_parsing_checked(parser, &|var, span|
						{
							match var
							{
								&ExpressionVariable::Global(ref name) =>
									if name == "pc" || name == "_"
										{ Ok(()) }
									else
									{
										Err(Error::new_with_span(
											"invalid variable; use `_` as a the argument stand-in",
											span.clone()))
									},
										
								&ExpressionVariable::Local(_) =>
									Err(Error::new_with_span("invalid variable", span.clone()))
							}
						})))
					}
					else
						{ None };
				
				let param_index = rule.add_parameter(name, allow_unresolved, constraint);
				rule.pattern_segments.push(PatternSegment::Parameter(param_index));
				
				try!(parser.expect_operator("}"));
			}
			
			else if parser.current().is_any_operator()
			{
				let (op, _) = try!(parser.expect_any_operator());
				rule.pattern_segments.push(PatternSegment::Exact(op.to_string()));
			}
			
			else
				{ return Err(Error::new_with_span("expected pattern", parser.current().span.clone())); }
			
			
			if parser.current().is_operator("->")
				{ break; }
		}
		
		Ok(())
	}


	fn parse_production(&self, parser: &mut Parser, rule: &mut Rule) -> Result<(), Error>
	{
		let begin_span = parser.current().span.clone();
		
		loop
		{
			let expr = try!(Expression::new_by_parsing_checked(parser, &|var, span|
			{
				match var
				{
					&ExpressionVariable::Global(ref name) =>
						if name == "pc" || rule.check_parameter_exists(name)
							{ Ok(()) }
						else
							{ Err(Error::new_with_span(format!("unknown parameter `{}`", name), span.clone())) },
							
					&ExpressionVariable::Local(_) =>
						Err(Error::new_with_span("invalid variable", span.clone()))
				}
			}));
			
			rule.production_bit_num += match expr.get_explicit_bit_num()
			{
				Some(bit_num) => bit_num,
				None => return Err(Error::new_with_span("expression has no explicit size; use bit slices", expr.span.clone()))
			};
			
			rule.production_segments.push(expr);
			
			if parser.current().is_linebreak_or_end()
				{ break; }
		}
		
		if rule.production_bit_num % self.align_bits != 0
		{
			let full_span = begin_span.join(&parser.current().span);
			return Err(Error::new_with_span(format!("production is not aligned to `{}` bits", self.align_bits), full_span));
		}
		
		Ok(())
	}
}