use syntax::{Token, TokenKind};


#[derive(Debug)]
pub struct Rule
{
	pub pattern_parts: Vec<RulePatternPart>,
	pub params: Vec<RuleParameter>
}


#[derive(Debug)]
pub enum RulePatternPart
{
	Exact(TokenKind, Option<String>),
	Parameter(usize)
}


#[derive(Debug)]
pub struct RuleParameter
{
	pub name: String,
	pub cascadable: bool
}


impl Rule
{
	pub fn new() -> Rule
	{
		Rule
		{
			pattern_parts: Vec::new(),
			params: Vec::new()
		}
	}
	
	
	pub fn pattern_add_exact(&mut self, token: &Token)
	{
		let part = RulePatternPart::Exact(token.kind, token.excerpt.clone());
		self.pattern_parts.push(part);
	}
	
	
	pub fn pattern_add_param<S>(&mut self, name: S, cascadable: bool)
	where S: Into<String>
	{
		let name_owned = name.into();
		
		assert!(!self.param_exists(&name_owned));
		
		let param_index = self.params.len();
		
		let param = RuleParameter
		{
			name: name_owned,
			cascadable: cascadable
		};
		self.params.push(param);
		
		let part = RulePatternPart::Parameter(param_index);
		self.pattern_parts.push(part);
	}
	
	
	pub fn param_exists(&self, name: &str) -> bool
	{
		self.params.iter().any(|p| p.name == name)
	}
}