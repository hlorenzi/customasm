use diagn::{Span, Report};
use expr::{Expression, ExpressionType, ExpressionValue};
use instrset::{InstrSet, Rule};
use asm::{AssemblerParser, BinaryOutput, RulePatternMatcher, LabelManager, LabelContext};
use util::FileServer;
use num::bigint::ToBigInt;



pub struct AssemblerState<'a>
{
	pub fileserver: &'a FileServer,
	pub instrset: &'a InstrSet,
	pub pattern_matcher: RulePatternMatcher,
	pub labels: LabelManager,
	pub parsed_instrs: Vec<ParsedInstruction>,
	pub bin_output: BinaryOutput,
	
	pub cur_address: usize,
	pub cur_writehead: usize
}


pub struct ParsedInstruction
{
	pub rule_index: usize,
	pub ctx: ExpressionContext,
	pub span: Span,
	pub exprs: Vec<Expression>,
	pub args: Vec<Option<ExpressionValue>>
}


pub struct ExpressionContext
{
	pub label_ctx: LabelContext,
	pub address: usize,
	pub writehead: usize
}


pub fn assemble<S>(report: &mut Report, instrset: &InstrSet, fileserver: &FileServer, filename: S) -> Result<BinaryOutput, ()>
where S: Into<String>
{
	let pattern_matcher = RulePatternMatcher::new(&instrset.rules);
	
	let mut state = AssemblerState
	{
		fileserver: fileserver,
		instrset: instrset,
		pattern_matcher: pattern_matcher,
		labels: LabelManager::new(),
		parsed_instrs: Vec::new(),
		bin_output: BinaryOutput::new(),
		
		cur_address: 0,
		cur_writehead: 0
	};
	
	AssemblerParser::parse_file(report, &mut state, filename)?;
	state.resolve_instrs(report)?;
	
	match report.has_errors()
	{
		true => Err(()),
		false => Ok(state.bin_output)
	}
}


impl<'a> AssemblerState<'a>
{
	pub fn get_cur_context(&self) -> ExpressionContext
	{
		ExpressionContext
		{
			label_ctx: self.labels.get_cur_context(),
			address: self.cur_address,
			writehead: self.cur_writehead
		}
	}

	pub fn resolve_instrs(&mut self, report: &mut Report) -> Result<(), ()>
	{
		use std::mem;
		
		let mut instrs = mem::replace(&mut self.parsed_instrs, Vec::new());
		
		for mut instr in &mut instrs
		{
			// Errors go to the report.
			let _ = self.output_instr(report, &mut instr);
		}
		
		mem::replace(&mut self.parsed_instrs, instrs);
		
		Ok(())
	}
	
	
	pub fn output_advance(&mut self, report: &mut Report, bytes: usize, span: &Span) -> Result<(), ()>
	{
		match self.cur_address.checked_add(bytes)
		{
			Some(addr) => self.cur_address = addr,
			None => return Err(report.error_span("address overflowed valid range", span))
		}
		
		match self.cur_writehead.checked_add(bytes)
		{
			Some(addr) => self.cur_writehead = addr,
			None => return Err(report.error_span("output pointer overflowed valid range", span))
		}
		
		Ok(())
	}
	
	
	pub fn output_zeroes(&mut self, report: &mut Report, bytes: usize, span: &Span) -> Result<(), ()>
	{
		let mut output_bit_index = self.cur_writehead * self.instrset.align;
		for _ in 0..bytes
		{
			for _ in 0..self.instrset.align
			{
				self.bin_output.write(output_bit_index, false);
				output_bit_index += 1;
			}
		}
		
		self.output_advance(report, bytes, span)
	}
	
	
	pub fn output_instr(&mut self, report: &mut Report, instr: &mut ParsedInstruction) -> Result<(), ()>
	{
		// Resolve remaining arguments.
		for i in 0..instr.exprs.len()
		{
			if instr.args[i].is_none()
				{ instr.args[i] = Some(self.expr_eval(report, &instr.ctx, &instr.exprs[i])?); }
		}
		
		// Check rule constraints.
		let rule = &self.instrset.rules[instr.rule_index];
		let get_arg = |i: usize| instr.args[i].clone();
		
		self.rule_check_all_constraints_satisfied(report, rule, &get_arg, &instr.ctx, &instr.span)?;
		
		// Output binary representation.
		let mut output_bit_index = instr.ctx.writehead * self.instrset.align;
		for part in &rule.production_parts
		{
			let (left, right) = part.slice().unwrap();
			let value = self.rule_expr_eval(report, rule, &get_arg, &instr.ctx, &part)?;
			
			for index in 0..(left - right + 1)
			{
				let bit = value.get_bit(left - right - index);
				self.bin_output.write(output_bit_index, bit);
				output_bit_index += 1;
			}
		}
		
		Ok(())
	}
	
	
	pub fn rule_check_all_constraints_satisfied<F>(&self, report: &mut Report, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, span: &Span) -> Result<(), ()>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		for constr in &rule.constraints
		{
			let satisfied = match self.rule_expr_eval(report, rule, get_arg, ctx, &constr.expr)?
			{
				ExpressionValue::Bool(b) => b,
				_ => unreachable!()
			};
			
			if !satisfied
			{
				match constr.descr
				{
					Some(ref descr) =>
						return Err(report.error_span(format!("constraint not satisfied: {}", descr), &span)),
					None =>
						return Err(report.error_span(format!("constraint not satisfied"), &span))
				}
			}
		}
		
		Ok(())
	}
	
	
	fn rule_expr_eval<F>(&self, report: &mut Report, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, expr: &Expression) -> Result<ExpressionValue, ()>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		expr.check_vars(&mut |name, span| self.rule_expr_check_var(report, rule, get_arg, name, span))?;
		expr.eval(report, &|name| self.rule_expr_get_var(rule, get_arg, ctx, name))
	}


	fn rule_expr_check_var<F>(&self, report: &mut Report, rule: &Rule, get_arg: &F, name: &str, span: &Span) -> Result<(), ()>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		if name == "pc"
			{ Ok (()) }
			
		else if rule.param_exists(name)
		{
			if get_arg(rule.param_index(name)).is_some()
				{ Ok(()) }
				
			else
				{ Err(report.error_span("unresolved argument", span)) }
		}
		
		else
			{ Err(report.error_span("unknown label", span)) }
	}
		
		
	fn rule_expr_get_var<F>(&self, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, name: &str) -> ExpressionValue
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		if name == "pc"
			{ ExpressionValue::Integer(ctx.address.to_bigint().unwrap()) }
		
		else
			{ get_arg(rule.param_index(name)).unwrap() }
	}
	
	
	pub fn expr_eval(&self, report: &mut Report, ctx: &ExpressionContext, expr: &Expression) -> Result<ExpressionValue, ()>
	{
		expr.check_vars(&mut |name, span| self.expr_check_var(report, ctx, name, span))?;
		
		if expr.eval_type(report, &|name| self.expr_get_var_type(name))? != ExpressionType::Integer
			{ return Err(report.error_span("expected integer value for instruction argument", &expr.span())); }
		
		expr.eval(report, &|name| self.expr_get_var(ctx, name))
	}


	fn expr_check_var(&self, report: &mut Report, ctx: &ExpressionContext, name: &str, span: &Span) -> Result<(), ()>
	{
		if name == "pc"
			{ Ok (()) }
			
		else if let Some('.') = name.chars().next()
		{
			if self.labels.local_exists(ctx.label_ctx, name)
				{ Ok(()) }
			else
				{ Err(report.error_span("unknown local label", span)) }
		}
		
		else if self.labels.global_exists(name)
			{ Ok(()) }
			
		else
			{ Err(report.error_span("unknown label", span)) }
	}
		
		
	fn expr_get_var_type(&self, _name: &str) -> ExpressionType
	{
		// All variables are integer type for now.
		ExpressionType::Integer
	}
		
		
	fn expr_get_var(&self, ctx: &ExpressionContext, name: &str) -> ExpressionValue
	{
		if name == "pc"
			{ ExpressionValue::Integer(ctx.address.to_bigint().unwrap()) }
		
		else if let Some('.') = name.chars().next()
			{ self.labels.get_local(ctx.label_ctx, name).unwrap().clone() }
		
		else
			{ self.labels.get_global(name).unwrap().clone() }
	}
}