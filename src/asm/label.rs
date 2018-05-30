use expr::ExpressionValue;
use num_bigint::BigInt;
use num_traits::Zero;
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
		let mut mngr = LabelManager
		{
			global_labels: Vec::new(),
			name_to_index_map: HashMap::new()
		};
		
		mngr.add_global("", ExpressionValue::Integer(BigInt::zero()));
		mngr
	}
	
	
	pub fn add_global<S>(&mut self, name: S, value: ExpressionValue)
	where S: Into<String>
	{
		self.name_to_index_map.insert(name.into(), self.global_labels.len());
		self.global_labels.push(GlobalLabel
		{
			value: value,
			local_labels: HashMap::new()
		});
	}
	
	
	pub fn add_local<S>(&mut self, ctx: LabelContext, name: S, value: ExpressionValue)
	where S: Into<String>
	{
		let global_label = &mut self.global_labels[ctx as usize];
		global_label.local_labels.insert(name.into(), value);
	}
	
	
	pub fn get_cur_context(&self) -> LabelContext
	{
		(self.global_labels.len() - 1) as LabelContext
	}
	
	
	pub fn get_global(&self, name: &str) -> Option<&ExpressionValue>
	{
		self.name_to_index_map.get(name).map(|index| &self.global_labels[*index].value)
	}
	
	
	pub fn get_local(&self, ctx: LabelContext, name: &str) -> Option<&ExpressionValue>
	{
		self.global_labels[ctx as usize].local_labels.get(name)
	}
	
	
	pub fn global_exists(&self, name: &str) -> bool
	{
		self.get_global(name).is_some()
	}
	
	
	pub fn local_exists(&self, ctx: LabelContext, name: &str) -> bool
	{
		self.get_local(ctx, name).is_some()
	}
}