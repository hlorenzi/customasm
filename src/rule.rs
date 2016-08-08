use util::expression::Expression;


pub struct Rule
{
	pub pattern_segments: Vec<PatternSegment>,
	pub pattern_params: Vec<Parameter>,
	pub production_segments: Vec<Expression>,
	pub production_bit_num: usize
}


pub enum PatternSegment
{
	Exact(String),
	Parameter(usize)
}


pub struct Parameter
{
	name: String,
	allow_unresolved: bool,
	constraint: Option<Expression>
}


impl Rule
{
	pub fn new() -> Rule
	{
		Rule
		{
			pattern_segments: Vec::new(),
			pattern_params: Vec::new(),
			production_segments: Vec::new(),
			production_bit_num: 0
		}
	}
	
	
	pub fn add_parameter(&mut self, name: String, allow_unresolved: bool, constraint: Option<Expression>) -> usize
	{
		debug_assert!(!self.check_parameter_exists(&name));
		
		self.pattern_params.push(Parameter
		{
			name: name,
			allow_unresolved: allow_unresolved,
			constraint: constraint
		});
		
		self.pattern_params.len() - 1
	}
	
	
	pub fn get_parameter(&self, name: &str) -> Option<usize>
	{
		match self.pattern_params.iter().enumerate().find(|&(_, param)| param.name == name)
		{
			Some((i, _)) => Some(i),
			None => None
		}
	}
	
	
	pub fn check_parameter_exists(&self, name: &str) -> bool
	{
		self.get_parameter(name).is_some()
	}
	
	
	pub fn get_parameter_allow_unresolved(&self, index: usize) -> bool
	{
		self.pattern_params[index].allow_unresolved
	}
	
	
	pub fn get_parameter_constraint(&self, index: usize) -> &Option<Expression>
	{
		&self.pattern_params[index].constraint
	}
}