use crate::*;
use std::collections::HashSet;


pub struct State
{
	pub banks: Vec<asm::Bank>,
	pub rule_groups: Vec<asm::Ruleset>,
	pub active_rule_groups: HashSet<RulesetRef>,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RulesetRef
{
	pub index: usize,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RuleRef
{
	pub ruleset_ref: RulesetRef,
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
			let _guard = report.push_parent("failed to resolve instruction", &rule_invokation.span);

			let candidate = &rule_invokation.candidates[0];
			let resolved = self.resolve_rule_invokation_candidate(
				report.clone(),
				&candidate)?;
			
			if let expr::Value::Integer(bigint) = resolved
			{
				bitvec.write_bigint(rule_invokation.bit_offset, bigint);
			}
		}

		Ok(bitvec)
	}


	pub fn resolve_rule_invokation_candidate(
		&self,
		report: diagn::RcReport,
		candidate: &asm::RuleInvokationCandidate)
		-> Result<expr::Value, ()>
	{
		let rule = self.get_rule(candidate.rule_ref).unwrap();

		let mut eval_ctx = expr::EvalContext::new();
		for (arg_index, arg) in candidate.args.iter().enumerate()
		{
			match arg
			{
				&asm::RuleInvokationArgument::Expression(ref expr) =>
				{
					let arg_value = expr.eval(
						report.clone(), &mut expr::EvalContext::new(),
						&|_| Err(false),
						&|_| Err(false))?;

					let arg_name = &rule.parameters[arg_index].name;

					eval_ctx.set_local(arg_name, arg_value);
				}

				&asm::RuleInvokationArgument::RuleGroup(ref inner_candidates) =>
				{
					let arg_value = self.resolve_rule_invokation_candidate(
						report.clone(),
						&inner_candidates[0])?;

					let arg_name = &rule.parameters[arg_index].name;

					eval_ctx.set_local(arg_name, arg_value);
				}
			}
		}
		
		rule.production.eval(
			report.clone(),
			&mut eval_ctx,
			&|_| Err(false),
			&|_| Err(false))
	}
	

	pub fn find_ruleset<TName: std::borrow::Borrow<str>>(
		&self,
		name: TName,
		report: diagn::RcReport,
		span: &diagn::Span)
		-> Result<RulesetRef, ()>
	{
		match self.rule_groups.iter().position(|rg| rg.name == name.borrow())
		{
			Some(index) => Ok(RulesetRef{ index }),
			None =>
			{
				report.error_span("unknown ruleset", span);
				Err(())
			}
		}
	}
	

	pub fn activate_ruleset<TName: std::borrow::Borrow<str>>(
		&mut self,
		name: TName,
		report: diagn::RcReport,
		span: &diagn::Span)
		-> Result<(), ()>
	{
		let rg_ref = self.find_ruleset(name.borrow(), report, span)?;

		self.active_rule_groups.insert(rg_ref);
		Ok(())
	}
	

	pub fn get_rule(
		&self,
		rule_ref: asm::RuleRef)
		-> Option<&asm::Rule>
	{
		Some(&self.rule_groups[rule_ref.ruleset_ref.index].rules[rule_ref.index])
	}
}