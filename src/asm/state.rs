use crate::*;
use std::collections::HashMap;
use std::collections::HashSet;


pub struct State
{
	pub rule_groups: HashMap<String, asm::RuleGroup>,
	pub active_rule_groups: HashSet<String>,
}


impl State
{
	pub fn new() -> State
	{
		let state = State
		{
			rule_groups: HashMap::new(),
			active_rule_groups: HashSet::new(),
		};
		
		state
	}
	
	
	pub fn process_file<S: Into<String>>(
        &mut self,
        report: diagn::RcReport,
        fileserver: &dyn util::FileServer,
        filename: S)
        -> Result<(), ()>
	{
        asm::parser::parse_file(report.clone(), self, fileserver, filename)?;
		
		match report.has_errors()
		{
			true => Err(()),
			false => Ok(())
		}
	}
	

	pub fn get_rule_group<TName: std::borrow::Borrow<str>>(
		&self,
		name: TName,
		report: diagn::RcReport,
		span: &diagn::Span)
		-> Result<&asm::RuleGroup, ()>
	{
		match self.rule_groups.get(name.borrow())
		{
			Some(rule_group) => Ok(rule_group),
			None =>
			{
				report.error_span("unknown rule group", span);
				Err(())
			}
		}
	}
	

	pub fn activate_rule_group<TName: Into<String> + std::borrow::Borrow<str>>(
		&mut self,
		name: TName,
		report: diagn::RcReport,
		span: &diagn::Span)
		-> Result<(), ()>
	{
		let _ = self.get_rule_group(name.borrow(), report, span)?;

		self.active_rule_groups.insert(name.into());
		Ok(())
	}
}