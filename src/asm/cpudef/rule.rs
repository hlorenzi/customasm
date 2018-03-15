use diagn::Span;
use syntax::{Token, TokenKind};
use expr::{Expression, ExpressionValue};


#[derive(Debug)]
pub struct Rule
{
	pub pattern_parts: Vec<RulePatternPart>,
	pub params: Vec<RuleParameter>,
	pub constraints: Vec<RuleConstraint>,
	pub production: Expression
}


#[derive(Debug, Eq, PartialEq, Hash)]
pub enum RulePatternPart
{
	Exact(TokenKind, Option<String>),
	Parameter(usize)
}


#[derive(Debug)]
pub struct RuleParameter
{
	pub name: String
}


#[derive(Debug)]
pub struct RuleConstraint
{
	pub expr: Expression,
	pub descr: Option<String>
}


impl Rule
{
	pub fn new() -> Rule
	{
		Rule
		{
			pattern_parts: Vec::new(),
			params: Vec::new(),
			constraints: Vec::new(),
			production: Expression::Literal(Span::new_dummy(), ExpressionValue::Bool(false))
		}
	}
	
	
	pub fn pattern_add_exact(&mut self, token: &Token)
	{
		let part = RulePatternPart::Exact(token.kind, token.excerpt.clone());
		self.pattern_parts.push(part);
	}
	
	
	pub fn pattern_add_param<S>(&mut self, name: S)
	where S: Into<String>
	{
		let name_owned = name.into();
		
		assert!(!self.param_exists(&name_owned));
		
		let param_index = self.params.len();
		
		let param = RuleParameter
		{
			name: name_owned
		};
		
		self.params.push(param);
		
		let part = RulePatternPart::Parameter(param_index);
		self.pattern_parts.push(part);
	}
	
	
	pub fn param_exists(&self, name: &str) -> bool
	{
		self.params.iter().any(|p| p.name == name)
	}
	
	
	pub fn param_index(&self, name: &str) -> usize
	{
		self.params.iter().enumerate().find(|p| p.1.name == name).unwrap().0
	}
	
	
	pub fn constraint_add(&mut self, expr: Expression, descr: Option<String>)
	{
		let constr = RuleConstraint
		{
			expr: expr,
			descr: descr
		};
		
		self.constraints.push(constr);
	}
}