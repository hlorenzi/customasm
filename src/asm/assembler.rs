use diagn::{Span, Message, Reporter};
use expr::{Expression, ExpressionType, ExpressionValue};
use instrset::{InstrSet, Rule};
use asm::{AssemblerParser, RulePatternMatcher, LabelManager, LabelContext};
use util::FileServer;
use num::bigint::ToBigInt;



pub struct AssemblerState<'a>
{
	pub reporter: &'a mut Reporter,
	pub fileserver: &'a FileServer,
	pub instrset: &'a InstrSet,
	pub pattern_matcher: RulePatternMatcher,
	pub labels: LabelManager,
	pub parsed_instrs: Vec<ParsedInstruction>,
	
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


pub fn assemble<S>(reporter: &mut Reporter, instrset: &InstrSet, fileserver: &FileServer, filename: S) -> Option<()>
where S: Into<String>
{
	let pattern_matcher = RulePatternMatcher::new(&instrset.rules);
	
	let mut state = AssemblerState
	{
		reporter: reporter,
		fileserver: fileserver,
		instrset: instrset,
		pattern_matcher: pattern_matcher,
		labels: LabelManager::new(),
		parsed_instrs: Vec::new(),
		
		cur_address: 0,
		cur_writehead: 0
	};
	
	if let Err(msg) = AssemblerParser::parse_file(&mut state, filename)
	{
		state.reporter.message(msg);
		return None;
	};
	
	if let Err(_) = state.resolve_instrs()
		{ return None; }
	
	if state.reporter.has_errors()
		{ return None; }
	
	Some(())
}


impl<'a> AssemblerState<'a>
{
	pub fn resolve_instrs(&mut self) -> Result<(), ()>
	{
		use std::mem;
		
		let mut instrs = mem::replace(&mut self.parsed_instrs, Vec::new());
		
		for mut instr in &mut instrs
		{
			// Errors go to the reporter.
			let _ = self.output_instr(&mut instr);
		}
		
		mem::replace(&mut self.parsed_instrs, instrs);
		
		Ok(())
	}
	
	
	pub fn output_instr(&mut self, instr: &mut ParsedInstruction) -> Result<(), ()>
	{
		// Resolve remaining arguments, and report errors.
		for i in 0..instr.exprs.len()
		{
			if instr.args[i].is_none()
			{
				match self.expr_eval(&instr.ctx, &instr.exprs[i])
				{
					Ok(arg) => instr.args[i] = Some(arg),
					Err(msg) =>
					{
						self.reporter.message(msg);
						return Err(());
					}
				}
			}
		}
		
		// Check rule constraints, and report errors.
		let rule = &self.instrset.rules[instr.rule_index];
		let get_arg = |i: usize| instr.args[i].clone();
		if let Err(msg) = self.rule_check_all_constraints_satisfied(rule, &get_arg, &instr.ctx, &instr.span)
		{
			self.reporter.message(msg);
			return Err(());
		}
		
		println!("");
		println!("output rule {}", instr.rule_index);
		println!("address 0x{:x}", instr.ctx.address);
		println!("args:");
		for expr in &instr.exprs
		{
			println!("  {:?}", self.expr_eval(&instr.ctx, expr).ok());
		}
		
		Ok(())
	}
	
	
	pub fn rule_check_all_constraints_satisfied<F>(&self, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, span: &Span) -> Result<(), Message>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		for constr in &rule.constraints
		{
			let satisfied = self.constraint_eval(rule, get_arg, ctx, &constr.expr)?;
			
			if !satisfied
			{
				match constr.descr
				{
					Some(ref descr) =>
						return Err(Message::error_span(format!("constraint not satisfied: {}", descr), &span)),
					None =>
						return Err(Message::error_span(format!("constraint not satisfied"), &span))
				}
			}
		}
		
		Ok(())
	}
	
	
	fn constraint_eval<F>(&self, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, constr: &Expression) -> Result<bool, Message>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		constr.check_vars(&|name, span| self.constraint_check_var(rule, get_arg, name, span))?;
		match constr.eval(&|name| self.constraint_get_var(rule, get_arg, ctx, name))?
		{
			ExpressionValue::Bool(b) => Ok(b),
			_ => unreachable!()
		}
	}


	fn constraint_check_var<F>(&self, rule: &Rule, get_arg: &F, name: &str, span: &Span) -> Result<(), Message>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		if name == "pc"
			{ Ok (()) }
			
		else if rule.param_exists(name)
		{
			if get_arg(rule.param_index(name)).is_some()
				{ Ok(()) }
				
			else
				{ Err(Message::error_span("unresolved argument", span)) }
		}
		
		else
			{ Err(Message::error_span("unknown label", span)) }
	}
		
		
	fn constraint_get_var<F>(&self, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, name: &str) -> ExpressionValue
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		if name == "pc"
			{ ExpressionValue::Integer(ctx.address.to_bigint().unwrap()) }
		
		else
			{ get_arg(rule.param_index(name)).unwrap() }
	}
	
	
	pub fn expr_eval(&self, ctx: &ExpressionContext, expr: &Expression) -> Result<ExpressionValue, Message>
	{
		expr.check_vars(&|name, span| self.expr_check_var(ctx, name, span))?;
		
		if expr.eval_type(&|name| self.expr_get_var_type(name))? != ExpressionType::Integer
			{ return Err(Message::error_span("expected integer value for instruction argument", &expr.span())); }
		
		expr.eval(&|name| self.expr_get_var(ctx, name))
	}


	fn expr_check_var(&self, ctx: &ExpressionContext, name: &str, span: &Span) -> Result<(), Message>
	{
		if name == "pc"
			{ Ok (()) }
			
		else if let Some('.') = name.chars().next()
		{
			if self.labels.does_local_exist(ctx.label_ctx, name)
				{ Ok(()) }
			else
				{ Err(Message::error_span("unknown local label", span)) }
		}
		
		else if self.labels.does_global_exist(name)
			{ Ok(()) }
			
		else
			{ Err(Message::error_span("unknown label", span)) }
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