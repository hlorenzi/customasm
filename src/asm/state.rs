use crate::*;
use std::collections::HashSet;


pub struct State
{
	pub banks: Vec<asm::Bank>,
	pub rule_groups: Vec<asm::RuleGroup>,
	pub active_rule_groups: HashSet<RuleGroupRef>,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RuleGroupRef
{
	pub index: usize,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RuleRef
{
	pub rule_group_ref: RuleGroupRef,
	pub index: usize,
}


impl State
{
	pub fn new() -> State
	{
		let mut state = State
		{
			banks: Vec::new(),
			rule_groups: Vec::new(),
			active_rule_groups: HashSet::new(),
		};

		state.banks.push(asm::Bank
		{
			cur_bit_offset: 0,
			rule_invokations: Vec::new(),
		});
		
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


	pub fn resolve_bank(
		&self,
		report: diagn::RcReport,
		bank: &asm::Bank)
		-> Result<util::BitVec, ()>
	{
		let mut bitvec = util::BitVec::new();

		for rule_invokation in &bank.rule_invokations
		{
			let candidate = &rule_invokation.candidates[0];
			let rule = self.get_rule(candidate.rule_ref).unwrap();
			let resolved = rule.production.eval(
				report.clone(), &mut expr::ExpressionEvalContext::new(),
				&|_, _, _| Err(false),
				&|_, _, _, _| Err(false))?;
			
			if let expr::ExpressionValue::Integer(bigint) = resolved
			{
				bitvec.write_bigint(rule_invokation.bit_offset, bigint);
			}
		}

		Ok(bitvec)
	}
	

	pub fn find_rule_group<TName: std::borrow::Borrow<str>>(
		&self,
		name: TName,
		report: diagn::RcReport,
		span: &diagn::Span)
		-> Result<RuleGroupRef, ()>
	{
		match self.rule_groups.iter().position(|rg| rg.name == name.borrow())
		{
			Some(index) => Ok(RuleGroupRef{ index }),
			None =>
			{
				report.error_span("unknown rule group", span);
				Err(())
			}
		}
	}
	

	pub fn activate_rule_group<TName: std::borrow::Borrow<str>>(
		&mut self,
		name: TName,
		report: diagn::RcReport,
		span: &diagn::Span)
		-> Result<(), ()>
	{
		let rg_ref = self.find_rule_group(name.borrow(), report, span)?;

		self.active_rule_groups.insert(rg_ref);
		Ok(())
	}
	

	pub fn get_rule(
		&self,
		rule_ref: asm::RuleRef)
		-> Option<&asm::Rule>
	{
		Some(&self.rule_groups[rule_ref.rule_group_ref.index].rules[rule_ref.index])
	}
}