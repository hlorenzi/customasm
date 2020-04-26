use crate::diagn::{Span, RcReport};
use crate::expr::{Expression, ExpressionValue, ExpressionEvalContext};
use crate::asm::{AssemblerParser, LabelManager, LabelContext};
use crate::asm::BankDef;
use crate::asm::Bank;
use crate::asm::BinaryBlock;
use crate::asm::cpudef::CpuDef;
use crate::asm::cpudef::RuleParameterType;
use crate::util::FileServer;
use num_bigint::ToBigInt;


pub struct AssemblerState
{
	pub cpudef: Option<CpuDef>,
	pub labels: LabelManager,
	pub parsed_instrs: Vec<ParsedInstruction>,
	pub parsed_exprs: Vec<ParsedExpression>,
	
	pub bankdefs: Vec<BankDef>,
	pub blocks: Vec<Bank>,
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


#[derive(Debug, Clone, PartialEq)]
pub struct BitRangeSpan
{
	pub start: usize,
	pub end: usize,
	pub addr: usize,
	pub span: Span
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
		
		state.bankdefs.push(BankDef::new("", 0, 0, Some(0), false, None));
		state.blocks.push(Bank::new(""));
		state
	}
	
	
	pub fn process_file<S>(&mut self, report: RcReport, fileserver: &dyn FileServer, filename: S) -> Result<(), ()>
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
		
		//for block in &self.blocks
		//	{ println!("{:#?}", block.bits.spans); }
		
		match report.has_errors()
		{
			true => Err(()),
			false => Ok(())
		}
	}
	
	
	pub fn get_binary_output(&self) -> BinaryBlock
	{
		let mut output = BinaryBlock::new();
		
		for block in &self.blocks
		{
			let bankdef_index = self.find_bankdef(&block.bank_name).unwrap();
			let bankdef = &self.bankdefs[bankdef_index];
			
			let bits = if bankdef_index == 0
				{ 1 }
			else
				{ self.cpudef.as_ref().unwrap().bits };
			
			if let Some(output_index) = bankdef.outp
			{
				let mut sorted_spans = block.bits.spans.clone();
				sorted_spans.sort_by(|a, b| a.start.cmp(&b.start));
				
				let mut sorted_span_index = 0;
				
				for i in 0..block.bits.len()
				{ 
					let bitrange_index = sorted_spans[sorted_span_index..].iter().position(|s| i >= s.start && i < s.end);
					let bitrange = bitrange_index.map(|i| &sorted_spans[sorted_span_index + i]);
					
					if let Some(bitrange_index) = bitrange_index
						{ sorted_span_index += bitrange_index; }
					
					if let Some(bitrange) = bitrange
						{ output.write(output_index * bits + i, block.bits.read(i), Some((bitrange.addr, &bitrange.span))); }
					else
						{ output.write(output_index * bits + i, block.bits.read(i), None); }
				}
				
				for bitrange in block.bits.spans.iter().filter(|s| s.start == s.end)
					{ output.mark_label(output_index * bits + bitrange.start, bitrange.addr, &bitrange.span); }
				
				if bankdef.fill
				{
					for i in block.bits.len()..(bankdef.size * bits)
						{ output.write(output_index * bits + i, false, None); }
				}
			}
		}
		
		output
	}


	pub fn get_symbol_output(&self) -> String
	{
		let mut result = String::new();

		for global_label in &self.labels.name_to_index_map
		{
			let global_value = &self.labels.global_labels[*global_label.1];

			if global_label.0 != ""
			{
				match global_value.value
				{
					ExpressionValue::Integer(ref integer) =>
					{
						result.push_str(&format!("{} = {:#x}\n", global_label.0, integer));
					}
					_ => {}
				}
			}

			for local_label in &global_value.local_labels
			{
				match local_label.1
				{
					ExpressionValue::Integer(ref integer) =>
					{
						result.push_str(&format!("{}{} = {:#x}\n", global_label.0, local_label.0, integer));
					}
					_ => {}
				}
			}
		}

		result
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
			offset: block.bits.len(),
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
		
		let bits = self.cpudef.as_ref().unwrap().bits;
		let block = &self.blocks[self.cur_block];
		
		let excess_bits = block.bits.len() % bits;
		if excess_bits != 0
		{
			let bits_short = bits - excess_bits;
			let plural = if bits_short > 1 { "bits" } else { "bit" };
			return Err(report.error_span(format!("address is not aligned to a word boundary ({} {} short)", bits_short, plural), span));
		}
			
		let bankdef_index = self.find_bankdef(&block.bank_name).unwrap();
		let bankdef = &self.bankdefs[bankdef_index];
		
		let block_offset = block.bits.len() / bits;
		let addr = match block_offset.checked_add(bankdef.addr)
		{
			Some(addr) => addr,
			None => return Err(report.error_span("address overflowed valid range", span))
		};
		
		if bankdef_index != 0 && addr >= bankdef.addr + bankdef.size
			{ return Err(report.error_span("address is out of bank range", span)); }
			
		Ok(addr)
	}
	
	
	fn get_bitrange_address(&self, report: RcReport, span: &Span) -> Result<usize, ()>
	{
		if self.cpudef.is_none()
			{ return Ok(0); }
		
		let bits = self.cpudef.as_ref().unwrap().bits;
		let block = &self.blocks[self.cur_block];
		
		let bankdef_index = self.find_bankdef(&block.bank_name).unwrap();
		let bankdef = &self.bankdefs[bankdef_index];
		
		let block_offset = block.bits.len() / bits;
		let addr = match block_offset.checked_add(bankdef.addr)
		{
			Some(addr) => addr,
			None => return Err(report.error_span("address overflowed valid range", span))
		};

		if bankdef_index != 0 && addr >= bankdef.addr + bankdef.size
			{ return Err(report.error_span("address is out of bank range", span)); }
		
		Ok(addr)
	}
	
	
	pub fn check_valid_address(&self, report: RcReport, block_index: usize, addr: usize, span: &Span) -> Result<(), ()>
	{
		let block = &self.blocks[block_index];
		let bankdef_index = self.find_bankdef(&block.bank_name).unwrap();
		let bankdef = &self.bankdefs[bankdef_index];
		
		if bankdef_index == 0
			{ return Ok(()); }
		
		if addr < bankdef.addr || addr > bankdef.addr + bankdef.size
			{ return Err(report.error_span("address is out of bank range", span)); }
			
		Ok(())
	}
	
	
	pub fn output_bits_until_aligned(&mut self, report: RcReport, multiple_of: usize, report_span: &Span, output_span: Option<&Span>) -> Result<(), ()>
	{
		if multiple_of == 0
			{ return Err(report.error_span("invalid alignment", report_span)); }
		
		self.check_cpudef_active(report.clone(), report_span)?;
		
		let bits = self.cpudef.as_ref().unwrap().bits;
		
		while self.blocks[self.cur_block].bits.len() % (bits * multiple_of) != 0
			{ self.output_bit(report.clone(), false, true, report_span, output_span)?; }
			
		Ok(())
	}
	
	
	pub fn output_bit(&mut self, report: RcReport, bit: bool, skipping: bool, report_span: &Span, output_span: Option<&Span>) -> Result<(), ()>
	{
		{
			let block = &self.blocks[self.cur_block];
			let bankdef = &self.bankdefs[self.cur_bank];
			
			if bankdef.outp.is_none() && !skipping
				{ return Err(report.error_span("attempt to place data in non-writable bank", report_span)); }
			
			if self.cur_bank != 0
			{
				self.check_cpudef_active(report.clone(), report_span)?;
				
				if block.bits.len() / self.cpudef.as_ref().unwrap().bits >= bankdef.size
					{ return Err(report.error_span("data overflowed bank size", report_span)); }
			}
		}
		
		let bitrange = match output_span
		{
			Some(output_span) =>
			{
				let addr = self.get_bitrange_address(report, output_span)?;
				Some((addr, output_span))
			}
			None => None
		};
		
		self.blocks[self.cur_block].bits.append(bit, bitrange);
		Ok(())
	}
	
	
	pub fn output_zero_bits(&mut self, report: RcReport, num: usize, skipping: bool, report_span: &Span, output_span: Option<&Span>) -> Result<(), ()>
	{
		for _ in 0..num
			{ self.output_bit(report.clone(), false, skipping, report_span, output_span)?; }
			
		Ok(())
	}
	
	
	pub fn mark_label(&mut self, addr: usize, output_span: &Span)
	{
		let index = self.blocks[self.cur_block].bits.len();
		self.blocks[self.cur_block].bits.mark_label(index, addr, output_span);
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


	pub fn check_expr_constraint(&self, report: RcReport, value: &ExpressionValue, typ: &RuleParameterType, span: &Span) -> Result<(), ()>
	{
		use crate::expr::bigint_bits;

		if let RuleParameterType::Unsigned(width) = typ
		{
			if let ExpressionValue::Integer(value_int) = value
			{
				if value_int.sign() == num_bigint::Sign::Minus ||
					bigint_bits(&value_int) > *width
					{ return Err(report.error_span(&format!("argument out of range for type `u{}`", width), &span)); }
			}
			else
				{ return Err(report.error_span(&format!("wrong argument for type `u{}`", width), &span)); }
		}
		else if let RuleParameterType::Signed(width) = typ
		{
			if let ExpressionValue::Integer(value_int) = value
			{
				if (value_int.sign() == num_bigint::Sign::NoSign && *width == 0) ||
					(value_int.sign() == num_bigint::Sign::Plus && bigint_bits(&value_int) >= *width) ||
					(value_int.sign() == num_bigint::Sign::Minus && bigint_bits(&value_int) > *width)
					{ return Err(report.error_span(&format!("argument out of range for type `s{}`", width), &span)); }
			}
			else
				{ return Err(report.error_span(&format!("wrong argument for type `s{}`", width), &span)); }
		}
		else if let RuleParameterType::Integer(width) = typ
		{
			if let ExpressionValue::Integer(value_int) = value
			{
				if bigint_bits(&value_int) > *width
					{ return Err(report.error_span(&format!("argument out of range for type `i{}`", width), &span)); }
			}
			else
				{ return Err(report.error_span(&format!("wrong argument for type `i{}`", width), &span)); }
		}

		Ok(())
	}
	
	
	pub fn output_parsed_instr(&mut self, report: RcReport, instr: &mut ParsedInstruction) -> Result<(), ()>
	{
		// Resolve remaining arguments.
		for i in 0..instr.exprs.len()
		{
			if instr.args[i].is_none()
				{ instr.args[i] = Some(self.expr_eval(report.clone(), &instr.ctx, &instr.exprs[i], &mut ExpressionEvalContext::new())?); }
		}
		
		// Check rule constraints.
		let rule = &self.cpudef.as_ref().unwrap().rules[instr.rule_index];
		let mut args_eval_ctx = ExpressionEvalContext::new();
		for i in 0..instr.args.len()
		{
			let arg = instr.args[i].clone().unwrap();
			self.check_expr_constraint(report.clone(), &arg, &rule.params[i].typ, &instr.exprs[i].span())?;
			args_eval_ctx.set_local(rule.params[i].name.clone(), arg);
		}
		
		// Output binary representation.
		let (left, right) = rule.production.slice().unwrap();
		
		let _guard = report.push_parent("failed to resolve instruction", &instr.span);
		
		let value = self.expr_eval(report.clone(), &instr.ctx, &rule.production, &mut args_eval_ctx)?;
		
		let addr = self.get_bitrange_address(report, &instr.span)?;
		
		let block = &mut self.blocks[instr.ctx.block];
		
		for i in 0..(left - right + 1)
		{
			let bit = value.get_bit(left - right - i);
			block.bits.write(instr.ctx.offset + i, bit, Some((addr, &instr.span)));
		}
		
		Ok(())
	}
	
	
	pub fn output_parsed_expr(&mut self, report: RcReport, expr: &ParsedExpression) -> Result<(), ()>
	{
		// Resolve expression.
		let value = self.expr_eval(report.clone(), &expr.ctx, &expr.expr, &mut ExpressionEvalContext::new())?;
		
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
			block.bits.write(expr.ctx.offset + i, bit, None);
		}
		
		Ok(())
	}
	
	
	pub fn expr_eval(&self, report: RcReport, ctx: &ExpressionContext, expr: &Expression, eval_ctx: &mut ExpressionEvalContext) -> Result<ExpressionValue, ()>
	{
		expr.eval(report.clone(), eval_ctx,
			&|report, name, span| self.expr_eval_var(report, ctx, name, span),
			&|report, fn_id, args, span| self.expr_eval_fn(report, fn_id, args, span))
	}
		
		
	fn expr_eval_var(&self, report: RcReport, ctx: &ExpressionContext, name: &str, span: &Span) -> Result<ExpressionValue, bool>
	{
		if name == "pc"
			{ Ok(ExpressionValue::Integer(ctx.get_address_at(report, self, span)?.to_bigint().unwrap())) }
			
		else if name == "assert"
			{ Ok(ExpressionValue::Function(0)) }
		
		else if let Some('.') = name.chars().next()
		{
			if self.labels.local_exists(ctx.label_ctx, name)
				{ Ok(self.labels.get_local(ctx.label_ctx, name).unwrap().clone()) }
			else
				{ Err(false) }
		}
		
		else
		{
			if self.labels.global_exists(name)
				{ Ok(self.labels.get_global(name).unwrap().clone()) }
			else
				{ Err(false) }
		}
	}
	
	
	fn expr_eval_fn(&self, report: RcReport, fn_id: usize, args: Vec<ExpressionValue>, span: &Span) -> Result<ExpressionValue, bool>
	{
		match fn_id
		{
			0 =>
			{
				if args.len() != 1
					{ return Err({ report.error_span("wrong number of arguments", span); true }); }
					
				match args[0]
				{
					ExpressionValue::Bool(value) =>
					{
						match value
						{
							true => Ok(ExpressionValue::Void),
							false => Err({ report.error_span("assertion failed", span); true })
						}
					}
					
					_ => Err({ report.error_span("wrong argument type", span); true })
				}
			}
			
			_ => unreachable!()
		}
	}
}


impl ExpressionContext
{
	pub fn get_address_at(&self, report: RcReport, state: &AssemblerState, span: &Span) -> Result<usize, bool>
	{
		if let Err(_) = state.check_cpudef_active(report.clone(), span)
			{ return Err(true); }
	
		let bits = state.cpudef.as_ref().unwrap().bits;
		let block = &state.blocks[self.block];
		
		if block.bits.len() % bits != 0
			{ return Err({ report.error_span("address is not aligned to a byte", span); true }); }
			
		let bankdef = state.find_bankdef(&block.bank_name).unwrap();
		
		let block_offset = self.offset / bits;
		match block_offset.checked_add(state.bankdefs[bankdef].addr)
		{
			Some(addr) => Ok(addr),
			None => Err({ report.error_span("address overflowed valid range", span); true })
		}
	}
}