use diagn::RcReport;
use super::Expression;
use super::ExpressionValue;
use super::UnaryOp;
use super::BinaryOp;
use num::BigInt;
use num::bigint::Sign;
use num::Zero;
use num::One;
use num::ToPrimitive;
use std::mem::swap;


impl Expression
{
	pub fn eval<F>(&self, report: RcReport, eval_var: &F) -> Result<ExpressionValue, ()>
	where F: Fn(&str) -> ExpressionValue
	{
		match self
		{
			&Expression::Literal(_, ref value) => Ok(value.clone()),
			
			&Expression::Variable(_, ref name) => Ok(eval_var(&name)),
			
			&Expression::UnaryOp(_, _, op, ref inner_expr) =>
			{
				match inner_expr.eval(report.clone(), eval_var)?
				{
					ExpressionValue::Integer(x) => match op
					{
						UnaryOp::Neg => Ok(ExpressionValue::Integer(-x)),
						UnaryOp::Not => Ok(ExpressionValue::Integer(bigint_not(x)))
					},
					ExpressionValue::Bool(b) => match op
					{
						UnaryOp::Not => Ok(ExpressionValue::Bool(!b)),
						_ => unreachable!()
					}
				}
			}
			
			&Expression::BinaryOp(_, ref op_span, op, ref lhs_expr, ref rhs_expr) =>
			{
				match (lhs_expr.eval(report.clone(), eval_var)?, rhs_expr.eval(report.clone(), eval_var)?)
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

							_ => unreachable!()
						}
					}
					
					(ExpressionValue::Bool(lhs), ExpressionValue::Bool(rhs)) =>
					{
						match op
						{
							BinaryOp::And |
							BinaryOp::LazyAnd => Ok(ExpressionValue::Bool(lhs & rhs)),
							BinaryOp::Or |
							BinaryOp::LazyOr  => Ok(ExpressionValue::Bool(lhs | rhs)),
							BinaryOp::Xor     => Ok(ExpressionValue::Bool(lhs ^ rhs)),
							BinaryOp::Eq      => Ok(ExpressionValue::Bool(lhs == rhs)),
							BinaryOp::Ne      => Ok(ExpressionValue::Bool(lhs != rhs)),
							_ => unreachable!()
						}
					}
					
					_ => unreachable!()
				}
			}
			
			&Expression::BitSlice(_, _, left, right, ref inner) =>
			{
				match inner.eval(report.clone(), eval_var)?
				{
					ExpressionValue::Integer(x) => Ok(ExpressionValue::Integer(bigint_slice(x, left, right))),
					_ => unreachable!()
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
	use num::Signed;
	bigint_or(lhs << rhs_width, rhs).abs()
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