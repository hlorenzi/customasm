use util::bigint::BigInt;
use util::expression::ExpressionValue;
use std::collections::HashMap;


pub struct LabelManager
{
	global_labels: Vec<GlobalLabel>,
	name_to_index_map: HashMap<String, usize>
}


pub type LabelContext = usize;


pub struct GlobalLabel
{
	value: ExpressionValue,
	local_labels: HashMap<String, ExpressionValue>
}


impl LabelManager
{
	pub fn new() -> LabelManager
	{
		let mut list = LabelManager
		{
			global_labels: Vec::new(),
			name_to_index_map: HashMap::new()
		};
		
		list.add_global("".to_string(), ExpressionValue::Integer(BigInt::from_usize(0)));
		list
	}
	
	
	pub fn add_global(&mut self, name: String, value: ExpressionValue)
	{
		self.name_to_index_map.insert(name, self.global_labels.len());
		self.global_labels.push(GlobalLabel
		{
			value: value,
			local_labels: HashMap::new()
		});
	}
	
	
	pub fn add_local(&mut self, ctx: LabelContext, name: String, value: ExpressionValue)
	{
		let global_label = &mut self.global_labels[ctx as usize];
		global_label.local_labels.insert(name, value);
	}
	
	
	pub fn get_cur_context(&self) -> LabelContext
	{
		(self.global_labels.len() - 1) as LabelContext
	}
	
	
	pub fn get_global(&self, name: &str) -> Option<&ExpressionValue>
	{
		match self.name_to_index_map.get(name)
		{
			Some(index) => Some(&self.global_labels[*index].value),
			None => None
		}
	}
	
	
	pub fn get_local(&self, ctx: LabelContext, name: &str) -> Option<&ExpressionValue>
	{
		match self.global_labels[ctx as usize].local_labels.get(name)
		{
			Some(value) => Some(value),
			None => None
		}
	}
	
	
	pub fn does_global_exist(&self, name: &str) -> bool
	{
		match self.name_to_index_map.get(name)
		{
			Some(_) => true,
			None => false
		}
	}
	
	
	pub fn does_local_exist(&self, ctx: LabelContext, name: &str) -> bool
	{
		match self.global_labels[ctx as usize].local_labels.get(name)
		{
			Some(_) => true,
			None => false
		}
	}
}