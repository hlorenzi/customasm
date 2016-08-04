use util::expression::Expression;


pub struct Rule
{
	pub pattern_segments: Vec<PatternSegment>,
	pub pattern_args: Vec<Argument>,
	pub production_segments: Vec<Expression>,
	pub production_bit_num: usize
}


pub enum PatternSegment
{
	Exact(String),
	Argument(usize)
}


pub struct Argument
{
	name: String,
	typ: VariableType
}


#[derive(Copy, Clone)]
pub struct VariableType
{
	pub bit_num: usize,
	pub signed: bool
}


impl Rule
{
	pub fn new() -> Rule
	{
		Rule
		{
			pattern_segments: Vec::new(),
			pattern_args: Vec::new(),
			production_segments: Vec::new(),
			production_bit_num: 0
		}
	}
	
	
	pub fn add_argument(&mut self, name: String, typ: VariableType) -> usize
	{
		assert!(!self.check_argument_exists(&name));
		self.pattern_args.push(Argument { name: name, typ: typ });
		self.pattern_args.len() - 1
	}
	
	
	pub fn get_argument(&self, name: &str) -> Option<usize>
	{
		match self.pattern_args.iter().enumerate().find(|&(_, arg)| arg.name == name)
		{
			Some((i, _)) => Some(i),
			None => None
		}
	}
	
	
	pub fn check_argument_exists(&self, name: &str) -> bool
	{
		self.get_argument(name).is_some()
	}
	
	
	pub fn get_argument_type(&self, index: usize) -> VariableType
	{
		self.pattern_args[index].typ
	}
}