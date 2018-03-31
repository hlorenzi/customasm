use diagn::{Span, RcReport};
use expr::{Expression, ExpressionType, ExpressionValue};
use asm::{AssemblerParser, BinaryOutput, LabelManager, LabelContext};
use asm::BankDef;
use asm::BinaryBlock;
use asm::cpudef::{CpuDef, Rule};
use util::FileServer;
use num::bigint::ToBigInt;


pub struct AssemblerState
{
	pub cpudef: Option<CpuDef>,
	pub labels: LabelManager,
	pub parsed_instrs: Vec<ParsedInstruction>,
	pub parsed_exprs: Vec<ParsedExpression>,
	
	pub bankdefs: Vec<BankDef>,
	pub blocks: Vec<BinaryBlock>,
	pub cur_bank: usize,
	pub cur_block: usize
}


pub struct ExpressionContext
{
	pub block: usize,
	pub offset: usize,
	pub label_ctx: LabelContext
}


pub struct ParsedInstruction
{
	pub rule_index: usize,
	pub ctx: ExpressionContext,
	pub span: Span,
	pub exprs: Vec<Expression>,
	pub args: Vec<Option<ExpressionValue>>
}


pub struct ParsedExpression
{
	pub ctx: ExpressionContext,
	pub width: usize,
	pub expr: Expression
}


impl AssemblerState
{
	pub fn new() -> AssemblerState
	{
		let mut state = AssemblerState
		{
			cpudef: None,
			labels: LabelManager::new(),
			parsed_instrs: Vec::new(),
			parsed_exprs: Vec::new(),
			
			bankdefs: Vec::new(),
			blocks: Vec::new(),
			cur_bank: 0,
			cur_block: 0
		};
		
		state.bankdefs.push(BankDef::new("", 0, 0x1_0000_0000, Some(0), false, None));
		state.blocks.push(BinaryBlock::new(""));
		state
	}
	
	
	pub fn process_file<S>(&mut self, report: RcReport, fileserver: &FileServer, filename: S) -> Result<(), ()>
	where S: Into<String>
	{
		AssemblerParser::parse_file(report.clone(), self, fileserver, filename, None)?;
		
		match report.has_errors()
		{
			true => Err(()),
			false => Ok(())
		}
	}
	
	
	pub fn wrapup(&mut self, report: RcReport) -> Result<(), ()>
	{
		self.resolve_instrs(report.clone())?;
		self.resolve_exprs(report.clone())?;
		self.check_bank_overlap(report.clone());
		
		match report.has_errors()
		{
			true => Err(()),
			false => Ok(())
		}
	}
	
	
	pub fn get_binary_output(&self) -> BinaryOutput
	{
		let mut output = BinaryOutput::new();
		
		for block in &self.blocks
		{
			let bankdef_index = self.find_bankdef(&block.bank_name).unwrap();
			let bankdef = &self.bankdefs[bankdef_index];
			
			let align = self.cpudef.as_ref().unwrap().align;
			
			if let Some(output_index) = bankdef.outp
			{
				for i in 0..block.len()
					{ output.write(output_index * align + i, block.read(i)); }
				
				if bankdef.fill
				{
					for i in block.len()..(bankdef.size * align)
						{ output.write(output_index * align + i, false); }
				}
			}
		}
		
		output
	}
}


impl AssemblerState
{
	pub fn check_cpudef_active(&self, report: RcReport, span: &Span) -> Result<(), ()>
	{
		if self.cpudef.is_none()
			{ Err(report.error_span("no cpu defined", span)) }
		else
			{ Ok(()) }
	}
	

	pub fn get_cur_context(&self) -> ExpressionContext
	{
		let block = &self.blocks[self.cur_block];
		
		ExpressionContext
		{
			block: self.cur_block,
			offset: block.len(),
			label_ctx: self.labels.get_cur_context()
		}
	}
	
	
	pub fn find_bankdef(&self, name: &str) -> Option<usize>
	{
		for i in 0..self.bankdefs.len()
		{
			if self.bankdefs[i].name == name
				{ return Some(i); }
		}
		
		None
	}
	
	
	pub fn check_bank_overlap(&self, report: RcReport)
	{
		for j in 1..self.bankdefs.len()
		{
			if self.bankdefs[j].outp.is_none()
				{ continue; }
		
			for i in 1..j
			{
				let bank1 = &self.bankdefs[i];
				let bank2 = &self.bankdefs[j];
				
				let outp1 = bank1.outp.unwrap();
				let outp2 = bank2.outp.unwrap();
				
				if outp1 + bank1.size > outp2 && outp1 < outp2 + bank2.size
					{ report.error_span(format!("banks `{}` and `{}` overlap in output location", bank1.name, bank2.name), &bank1.decl_span.as_ref().unwrap()); }
			}
		}
	}
	
	
	pub fn get_cur_address(&self, report: RcReport, span: &Span) -> Result<usize, ()>
	{
		self.check_cpudef_active(report.clone(), span)?;
		
		let align = self.cpudef.as_ref().unwrap().align;
		let block = &self.blocks[self.cur_block];
		
		let excess_bits = block.len() % align;
		if excess_bits != 0
		{
			let bits_short = align - excess_bits;
			let plural = if bits_short > 1 { "bits" } else { "bit" };
			return Err(report.error_span(format!("address is not aligned to a byte ({} {} short)", bits_short, plural), span));
		}
			
		let bankdef_index = self.find_bankdef(&block.bank_name).unwrap();
		let bankdef = &self.bankdefs[bankdef_index];
		
		let block_offset = block.len() / align;
		let addr = match block_offset.checked_add(bankdef.addr)
		{
			Some(addr) => addr,
			None => return Err(report.error_span("address overflowed valid range", span))
		};
		
		if addr >= bankdef.addr + bankdef.size
			{ return Err(report.error_span("address is out of bank range", span)); }
			
		Ok(addr)
	}
	
	
	pub fn check_valid_address(&self, report: RcReport, block_index: usize, addr: usize, span: &Span) -> Result<(), ()>
	{
		self.check_cpudef_active(report.clone(), span)?;
		
		let block = &self.blocks[block_index];
		let bankdef_index = self.find_bankdef(&block.bank_name).unwrap();
		let bankdef = &self.bankdefs[bankdef_index];
		
		if addr < bankdef.addr || addr > bankdef.addr + bankdef.size
			{ return Err(report.error_span("address is out of bank range", span)); }
			
		Ok(())
	}
	
	
	pub fn output_bit(&mut self, report: RcReport, bit: bool, skipping: bool, span: &Span) -> Result<(), ()>
	{
		self.check_cpudef_active(report.clone(), span)?;
		
		{
			let block = &self.blocks[self.cur_block];
			let bankdef = &self.bankdefs[self.cur_bank];
			
			if bankdef.outp.is_none() && !skipping
				{ return Err(report.error_span("attempt to place data in non-writable bank", span)); }
			
			if block.len() / self.cpudef.as_ref().unwrap().align >= bankdef.size
				{ return Err(report.error_span("data overflowed bank size", span)); }
		}
		
		self.blocks[self.cur_block].append(bit);
		Ok(())
	}
	
	
	pub fn output_zero_bits(&mut self, report: RcReport, num: usize, skipping: bool, span: &Span) -> Result<(), ()>
	{
		for _ in 0..num
			{ self.output_bit(report.clone(), false, skipping, span)?; }
			
		Ok(())
	}

	
	pub fn resolve_instrs(&mut self, report: RcReport) -> Result<(), ()>
	{
		use std::mem;
		
		let mut instrs = mem::replace(&mut self.parsed_instrs, Vec::new());
		
		for mut instr in &mut instrs
		{
			// Errors go to the report.
			let _ = self.output_parsed_instr(report.clone(), &mut instr);
		}
		
		mem::replace(&mut self.parsed_instrs, instrs);
		
		Ok(())
	}
	

	pub fn resolve_exprs(&mut self, report: RcReport) -> Result<(), ()>
	{
		use std::mem;
		
		let exprs = mem::replace(&mut self.parsed_exprs, Vec::new());
		
		for expr in &exprs
		{
			// Errors go to the report.
			let _ = self.output_parsed_expr(report.clone(), expr);
		}
		
		mem::replace(&mut self.parsed_exprs, exprs);
		
		Ok(())
	}
	
	
	pub fn output_parsed_instr(&mut self, report: RcReport, instr: &mut ParsedInstruction) -> Result<(), ()>
	{
		// Resolve remaining arguments.
		for i in 0..instr.exprs.len()
		{
			if instr.args[i].is_none()
				{ instr.args[i] = Some(self.expr_eval(report.clone(), &instr.ctx, &instr.exprs[i])?); }
		}
		
		// Check rule constraints.
		let rule = &self.cpudef.as_ref().unwrap().rules[instr.rule_index];
		let get_arg = |i: usize| instr.args[i].clone();
		
		self.rule_check_all_constraints_satisfied(report.clone(), rule, &get_arg, &instr.ctx, &instr.span)?;
		
		// Output binary representation.
		let (left, right) = rule.production.slice().unwrap();
		let value = self.rule_expr_eval(report.clone(), rule, &get_arg, &instr.ctx, &rule.production)?;
		
		let block = &mut self.blocks[instr.ctx.block];
		
		for i in 0..(left - right + 1)
		{
			let bit = value.get_bit(left - right - i);
			block.write(instr.ctx.offset + i, bit);
		}
		
		Ok(())
	}
	
	
	pub fn output_parsed_expr(&mut self, report: RcReport, expr: &ParsedExpression) -> Result<(), ()>
	{
		// Resolve expression.
		let value = self.expr_eval(report.clone(), &expr.ctx, &expr.expr)?;
		
		// Check size constraints.
		let value_width = value.bits();
		
		if value_width > expr.width
		{
			let descr = format!("value (width = {}) is larger than the specified width; use a bit slice", value_width);
			return Err(report.error_span(descr, &expr.expr.span()));
		}
		
		// Output binary representation.
		let block = &mut self.blocks[expr.ctx.block];
		
		for i in 0..expr.width
		{
			let bit = value.get_bit(expr.width - i - 1);
			block.write(expr.ctx.offset + i, bit);
		}
		
		Ok(())
	}
	
	
	pub fn rule_check_all_constraints_satisfied<F>(&self, report: RcReport, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, span: &Span) -> Result<(), ()>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		for constr in &rule.constraints
		{
			let satisfied = match self.rule_expr_eval(report.clone(), rule, get_arg, ctx, &constr.expr)?
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
	
	
	fn rule_expr_eval<F>(&self, report: RcReport, rule: &Rule, get_arg: &F, ctx: &ExpressionContext, expr: &Expression) -> Result<ExpressionValue, ()>
	where F: Fn(usize) -> Option<ExpressionValue>
	{
		expr.check_vars(&mut |name, span| self.rule_expr_check_var(report.clone(), rule, get_arg, name, span))?;
		expr.eval(report.clone(), &|name| self.rule_expr_get_var(rule, get_arg, ctx, name))
	}


	fn rule_expr_check_var<F>(&self, report: RcReport, rule: &Rule, get_arg: &F, name: &str, span: &Span) -> Result<(), ()>
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
			{ ExpressionValue::Integer(ctx.get_address_at(self).to_bigint().unwrap()) }
		
		else
			{ get_arg(rule.param_index(name)).unwrap() }
	}
	
	
	pub fn expr_eval(&self, report: RcReport, ctx: &ExpressionContext, expr: &Expression) -> Result<ExpressionValue, ()>
	{
		expr.check_vars(&mut |name, span| self.expr_check_var(report.clone(), ctx, name, span))?;
		
		if expr.eval_type(report.clone(), &|name| self.expr_get_var_type(name))? != ExpressionType::Integer
			{ return Err(report.error_span("expected integer value for instruction argument", &expr.span())); }
		
		expr.eval(report.clone(), &|name| self.expr_get_var(ctx, name))
	}


	fn expr_check_var(&self, report: RcReport, ctx: &ExpressionContext, name: &str, span: &Span) -> Result<(), ()>
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
			{ ExpressionValue::Integer(ctx.get_address_at(self).to_bigint().unwrap()) }
		
		else if let Some('.') = name.chars().next()
			{ self.labels.get_local(ctx.label_ctx, name).unwrap().clone() }
		
		else
			{ self.labels.get_global(name).unwrap().clone() }
	}
}


impl ExpressionContext
{
	pub fn get_address_at(&self, state: &AssemblerState) -> usize
	{
		let align = state.cpudef.as_ref().unwrap().align;
		let block = &state.blocks[self.block];
		
		if block.len() % align != 0
			{ panic!("address is not aligned to a byte"); }
			
		let bankdef = state.find_bankdef(&block.bank_name).unwrap();
		
		let block_offset = self.offset / align;
		match block_offset.checked_add(state.bankdefs[bankdef].addr)
		{
			Some(addr) => addr,
			None => panic!("address overflowed valid range")
		}
	}
}