use diagn::{RcReport, Span};
use super::Expression;
use super::ExpressionValue;
use super::UnaryOp;
use super::BinaryOp;
use num_bigint::BigInt;
use num_bigint::Sign;
use num_traits::Zero;
use num_traits::One;
use num_traits::ToPrimitive;
use std::mem::swap;
use std::collections::HashMap;


pub struct ExpressionEvalContext
{
	locals: HashMap<String, ExpressionValue>
}


impl ExpressionEvalContext
{
	pub fn new() -> ExpressionEvalContext
	{
		ExpressionEvalContext
		{
			locals: HashMap::new()
		}
	}
	
	
	pub fn set_local<S>(&mut self, name: S, value: ExpressionValue)
	where S: Into<String>
	{
		self.locals.insert(name.into(), value);
	}
	
	
	pub fn get_local(&self, name: &str) -> Result<ExpressionValue, ()>
	{
		match self.locals.get(name)
		{
			Some(value) => Ok(value.clone()),
			None => Err(())
		}
	}
}


impl Expression
{
	pub fn eval<FVar, FFn>(&self, report: RcReport, ctx: &mut ExpressionEvalContext, eval_var: &FVar, eval_fn: &FFn) -> Result<ExpressionValue, ()>
		where
		FVar: Fn(RcReport, &str, &Span) -> Result<ExpressionValue, bool>,
		FFn: Fn(RcReport, usize, Vec<ExpressionValue>, &Span) -> Result<ExpressionValue, bool>
	{
		match self
		{
			&Expression::Literal(_, ref value) => Ok(value.clone()),
			
			&Expression::Variable(ref span, ref name) => match ctx.get_local(&name)
			{
				Ok(value) => Ok(value),
				Err(_) => match eval_var(report.clone(), &name, &span)
				{
					Ok(value) => Ok(value),
					Err(handled) =>
					{
						if !handled
							{ report.error_span("unknown variable", &span); }
							
						Err(())
					}
				}
			}
			
			&Expression::UnaryOp(ref span, _, op, ref inner_expr) =>
			{
				match inner_expr.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Integer(x) => match op
					{
						UnaryOp::Neg => Ok(ExpressionValue::Integer(-x)),
						UnaryOp::Not => Ok(ExpressionValue::Integer(bigint_not(x)))
					},
					
					ExpressionValue::Bool(b) => match op
					{
						UnaryOp::Not => Ok(ExpressionValue::Bool(!b)),
						_ => Err(report.error_span("invalid argument type to operator", &span))
					},
					
					_ => Err(report.error_span("invalid argument type to operator", &span))
				}
			}
			
			&Expression::BinaryOp(ref span, ref op_span, op, ref lhs_expr, ref rhs_expr) =>
			{
				if op == BinaryOp::Assign
				{
					use std::ops::Deref;
					
					match lhs_expr.deref()
					{
						&Expression::Variable(_, ref name) =>
						{
							let value = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
							ctx.set_local(name.clone(), value);
							Ok(ExpressionValue::Void)
						}
						
						_ => Err(report.error_span("invalid assignment destination", &lhs_expr.span()))
					}
				}
				
				else if op == BinaryOp::LazyOr || op == BinaryOp::LazyAnd
				{
					let lhs = lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
					
					match (op, &lhs)
					{
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(true))  => return Ok(lhs),
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(false)) => return Ok(lhs),
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(false)) => { }
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(true))  => { }
						_ => return Err(report.error_span("invalid argument type to operator", &lhs_expr.span()))
					}
					
					let rhs = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
					
					match (op, &rhs)
					{
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(true))  => Ok(rhs),
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(false)) => Ok(rhs),
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(false)) => Ok(rhs),
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(true))  => Ok(rhs),
						_ => Err(report.error_span("invalid argument type to operator", &rhs_expr.span()))
					}
				}
				
				else
				{
					match (lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?, rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?)
					{
						(ExpressionValue::Integer(lhs), ExpressionValue::Integer(rhs)) =>
						{
							match op
							{
								BinaryOp::Add => Ok(ExpressionValue::Integer(lhs + rhs)),
								BinaryOp::Sub => Ok(ExpressionValue::Integer(lhs - rhs)),
								BinaryOp::Mul => Ok(ExpressionValue::Integer(lhs * rhs)),
								
								BinaryOp::Div => match lhs.checked_div(&rhs)
								{
									Some(x) => Ok(ExpressionValue::Integer(x)),
									None => Err(report.error_span("division by zero", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::Mod => match bigint_checked_rem(lhs, rhs)
								{
									Some(x) => Ok(ExpressionValue::Integer(x)),
									None => Err(report.error_span("modulo by zero", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::Shl => match bigint_shl(lhs, rhs)
								{
									Some(x) => Ok(ExpressionValue::Integer(x)),
									None => Err(report.error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::Shr => match bigint_shr(lhs, rhs)
								{
									Some(x) => Ok(ExpressionValue::Integer(x)),
									None => Err(report.error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::And  => Ok(ExpressionValue::Integer(bigint_and(lhs, rhs))),
								BinaryOp::Or   => Ok(ExpressionValue::Integer(bigint_or (lhs, rhs))),
								BinaryOp::Xor  => Ok(ExpressionValue::Integer(bigint_xor(lhs, rhs))),
								BinaryOp::Eq   => Ok(ExpressionValue::Bool(lhs == rhs)),
								BinaryOp::Ne   => Ok(ExpressionValue::Bool(lhs != rhs)),
								BinaryOp::Lt   => Ok(ExpressionValue::Bool(lhs <  rhs)),
								BinaryOp::Le   => Ok(ExpressionValue::Bool(lhs <= rhs)),
								BinaryOp::Gt   => Ok(ExpressionValue::Bool(lhs >  rhs)),
								BinaryOp::Ge   => Ok(ExpressionValue::Bool(lhs >= rhs)),
								
								BinaryOp::Concat =>
								{
									match (lhs_expr.width(), rhs_expr.width())
									{
										(Some(lhs_width), Some(rhs_width)) => Ok(ExpressionValue::Integer(bigint_concat(lhs, lhs_width, rhs, rhs_width))),
										(None, _) => Err(report.error_span("argument to concatenation with no known width", &lhs_expr.span())),
										(_, None) => Err(report.error_span("argument to concatenation with no known width", &rhs_expr.span()))
									}							
								}

								_ => Err(report.error_span("invalid argument types to operator", &span))
							}
						}
						
						(ExpressionValue::Bool(lhs), ExpressionValue::Bool(rhs)) =>
						{
							match op
							{
								BinaryOp::And => Ok(ExpressionValue::Bool(lhs & rhs)),
								BinaryOp::Or  => Ok(ExpressionValue::Bool(lhs | rhs)),
								BinaryOp::Xor => Ok(ExpressionValue::Bool(lhs ^ rhs)),
								BinaryOp::Eq  => Ok(ExpressionValue::Bool(lhs == rhs)),
								BinaryOp::Ne  => Ok(ExpressionValue::Bool(lhs != rhs)),
								_ => Err(report.error_span("invalid argument types to operator", &span))
							}
						}
						
						_ => Err(report.error_span("invalid argument types to operator", &span))
					}
				}
			}
			
			&Expression::TernaryOp(_, ref cond, ref true_branch, ref false_branch) =>
			{
				match cond.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Bool(true)  => true_branch.eval(report.clone(), ctx, eval_var, eval_fn),
					ExpressionValue::Bool(false) => false_branch.eval(report.clone(), ctx, eval_var, eval_fn),
					_ => Err(report.error_span("invalid condition type", &cond.span()))
				}
			}
			
			&Expression::BitSlice(ref span, _, left, right, ref inner) =>
			{
				match inner.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Integer(x) => Ok(ExpressionValue::Integer(bigint_slice(x, left, right))),
					_ => Err(report.error_span("invalid argument type to slice", &span))
				}
			}
			
			&Expression::Block(_, ref exprs) =>
			{
				let mut result = ExpressionValue::Void;
				
				for expr in exprs
					{ result = expr.eval(report.clone(), ctx, eval_var, eval_fn)?; }
					
				Ok(result)
			}
			
			&Expression::Call(ref span, ref target, ref arg_exprs) =>
			{
				match target.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Function(id) =>
					{
						let mut args = Vec::new();
						for expr in arg_exprs
							{ args.push(expr.eval(report.clone(), ctx, eval_var, eval_fn)?); }
							
						match eval_fn(report, id, args, &span)
						{
							Ok(value) => Ok(value),
							Err(_handled) => Err(())
						}
					}
					
					_ => Err(report.error_span("expression is not callable", &target.span()))
				}
			}
		}
	}
}


impl ExpressionValue
{
	pub fn bits(&self) -> usize
	{
		match self
		{
			&ExpressionValue::Integer(ref bigint) => bigint_bits(&bigint),
			
			_ => panic!("not an integer")
		}
	}
	

	pub fn get_bit(&self, index: usize) -> bool
	{
		match self
		{
			&ExpressionValue::Integer(ref bigint) =>
			{
				let bytes = bigint.to_signed_bytes_le();
				
				let byte_index = index / 8;
				if byte_index >= bytes.len()
					{ return bigint.sign() == Sign::Minus; }
					
				let mut byte = bytes[byte_index];
				
				let mut bit_index = index % 8;
				while bit_index > 0
				{
					byte >>= 1;
					bit_index -= 1;
				}
				
				(byte & 0b1) != 0
			}
			
			_ => panic!("not an integer")
		}
	}
}


fn bigint_bits(x: &BigInt) -> usize
{
	if x.is_zero()
		{ return 1; }

	if x < &BigInt::zero()
	{
		let y: BigInt = x + 1;
		y.bits() + 1
	}
	else
		{ x.bits() }
}


fn bigint_checked_rem(lhs: BigInt, rhs: BigInt) -> Option<BigInt>
{
	if rhs == BigInt::zero()
		{ None }
	else
		{ Some(lhs % rhs) }
}


fn bigint_shl(lhs: BigInt, rhs: BigInt) -> Option<BigInt>
{
	rhs.to_usize().map(|rhs| lhs << rhs)
}


fn bigint_shr(lhs: BigInt, rhs: BigInt) -> Option<BigInt>
{
	let lhs_sign = lhs.sign();
	
	match rhs.to_usize().map(|rhs| lhs >> rhs)
	{
		None => None,
		Some(result) =>
			if lhs_sign == Sign::Minus && result.sign() == Sign::NoSign
				{ Some(BigInt::from(-1)) }
			else
				{ Some(result) }
	}
}


fn bigint_concat(lhs: BigInt, _lhs_width: usize, rhs: BigInt, rhs_width: usize) -> BigInt
{
	bigint_or(lhs << rhs_width, rhs)
}


fn bigint_not(x: BigInt) -> BigInt
{
	let mut x_bytes = x.to_signed_bytes_le();
	
	for i in 0..x_bytes.len()
		{ x_bytes[i] = !x_bytes[i]; }
		
	BigInt::from_signed_bytes_le(&x_bytes)
}


fn bigint_bitmanipulate<F>(lhs: BigInt, rhs: BigInt, f: F) -> BigInt
where F: Fn(u8, u8) -> u8
{
	let mut lhs_bytes = lhs.to_signed_bytes_le();
	let mut lhs_sign = lhs.sign();
	let mut rhs_bytes = rhs.to_signed_bytes_le();
	let mut rhs_sign = rhs.sign();
	
	if lhs_sign != Sign::Minus && (lhs_bytes[0] & 0x80) != 0
		{ lhs_bytes.push(0); }
	
	if rhs_sign != Sign::Minus && (rhs_bytes[0] & 0x80) != 0
		{ rhs_bytes.push(0); }
		
	if rhs_bytes.len() > lhs_bytes.len()
	{
		swap(&mut lhs_bytes, &mut rhs_bytes);
		swap(&mut lhs_sign, &mut rhs_sign);
	}
	
	for i in 0..lhs_bytes.len()
	{
		let rhs_byte = if i < rhs_bytes.len()
			{ rhs_bytes[i] }
		else if rhs_sign == Sign::Minus
			{ 0xff }
		else
			{ 0 };
		
		lhs_bytes[i] = f(lhs_bytes[i], rhs_byte);
	}
	
	BigInt::from_signed_bytes_le(&lhs_bytes)
}


fn bigint_and(lhs: BigInt, rhs: BigInt) -> BigInt
{
	bigint_bitmanipulate(lhs, rhs, |a, b| a & b)
}


fn bigint_or(lhs: BigInt, rhs: BigInt) -> BigInt
{
	bigint_bitmanipulate(lhs, rhs, |a, b| a | b)
}


fn bigint_xor(lhs: BigInt, rhs: BigInt) -> BigInt
{
	bigint_bitmanipulate(lhs, rhs, |a, b| a ^ b)
}


fn bigint_slice(x: BigInt, left: usize, right: usize) -> BigInt
{
	let mut mask = BigInt::zero();
	for _ in 0..(left - right + 1)
		{ mask = (mask << 1) + BigInt::one(); }
	
	let shifted = bigint_shr(x, BigInt::from(right)).unwrap();
	
	bigint_and(shifted, mask)	
}