use crate::*;
use std::collections::HashSet;


pub struct State
{
	pub banks: Vec<asm::Bank>,
	pub symbols: asm::SymbolManager,
	pub rule_groups: Vec<asm::Ruleset>,
	pub active_rule_groups: HashSet<RulesetRef>,
}


#[derive(Clone, Debug)]
pub struct Context
{
	pub bit_offset: usize,
	pub symbol_ctx: asm::SymbolContext,
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
			symbols: asm::SymbolManager::new(),
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


	pub fn get_ctx(&self) -> Context
	{
        let bit_offset = self.banks[0].cur_bit_offset;
		let symbol_ctx = self.symbols.get_ctx();

		Context
		{
			bit_offset,
			symbol_ctx,
		}
	}
	
	
	pub fn get_addr(&self, report: diagn::RcReport, ctx: &Context, span: &diagn::Span) -> Result<util::BigInt, ()>
	{
		let bits = 8;
		let _bank = &self.banks[0];
		
		let excess_bits = ctx.bit_offset % bits;
		if excess_bits != 0
		{
			let bits_short = bits - excess_bits;
			let plural = if bits_short > 1 { "bits" } else { "bit" };
			return Err(report.error_span(format!("position is not aligned to an address boundary ({} {} short)", bits_short, plural), span));
		}
			
		let addr =
			&util::BigInt::from(ctx.bit_offset / bits) +
			&util::BigInt::from(0);
		
		Ok(addr)
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
				&rule_invokation,
				&candidate)?;
			
			if let expr::Value::Integer(bigint) = resolved
			{
				bitvec.write_bigint(rule_invokation.ctx.bit_offset, bigint);
			}
		}

		Ok(bitvec)
	}


	pub fn resolve_rule_invokation_candidate(
		&self,
		report: diagn::RcReport,
		invokation: &asm::RuleInvokation,
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
					let arg_value = self.eval_expr(
						report.clone(),
						&expr,
						&invokation.ctx,
						&mut expr::EvalContext::new())?;

					let arg_name = &rule.parameters[arg_index].name;

					eval_ctx.set_local(arg_name, arg_value);
				}

				&asm::RuleInvokationArgument::NestedRule(ref inner_candidates) =>
				{
					let arg_value = self.resolve_rule_invokation_candidate(
						report.clone(),
						invokation,
						&inner_candidates[0])?;

					let arg_name = &rule.parameters[arg_index].name;

					eval_ctx.set_local(arg_name, arg_value);
				}
			}
		}

		self.eval_expr(
			report,
			&rule.production,
			&invokation.ctx,
			&mut eval_ctx)
	}
	

	pub fn eval_expr(
		&self,
		report: diagn::RcReport,
		expr: &expr::Expr,
		ctx: &Context,
		eval_ctx: &mut expr::EvalContext)
		-> Result<expr::Value, ()>
	{
		expr.eval(
			report,
			eval_ctx,
			&|info| self.eval_var(ctx, info),
			&|_| Err(false))
	}
	
		
	fn eval_var(
		&self,
		ctx: &Context,
		info: &expr::EvalVariableInfo)
		-> Result<expr::Value, bool>
	{
		if info.hierarchy_level == 0 && info.hierarchy.len() == 1
		{
			match info.hierarchy[0].as_ref()
			{
				"$" | "pc" =>
				{
					return match self.get_addr(
						info.report.clone(),
						&ctx,
						&info.span)
					{
						Err(()) => Err(true),
						Ok(addr) => Ok(expr::Value::make_integer(addr))
					}
				}

				_ => {}
			}
		}

		//println!("reading hierarchy level {}, hierarchy {:?}, ctx {:?}", info.hierarchy_level, info.hierarchy, &ctx.symbol_ctx);

		if let Some(symbol) = self.symbols.get(&ctx.symbol_ctx, info.hierarchy_level, info.hierarchy)
		{
			Ok(symbol.value.clone())
		}
		else
		{
			Err(false)
		}
	}
}