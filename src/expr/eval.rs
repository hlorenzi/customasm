use diagn::Message;
use super::Expression;
use super::ExpressionValue;
use super::UnaryOp;
use super::BinaryOp;
use num::BigInt;
use num::Zero;
use num::traits::Signed;
use num::ToPrimitive;


impl Expression
{
	pub fn eval<F>(&self, eval_var: &F) -> Result<ExpressionValue, Message>
	where F: Fn(&str) -> ExpressionValue
	{
		match self
		{
			&Expression::Literal(_, ref value) => Ok(value.clone()),
			
			&Expression::Variable(_, ref name) => Ok(eval_var(&name)),
			
			&Expression::UnaryOp(_, ref op_span, op, ref inner_expr) =>
			{
				match inner_expr.eval(eval_var)?
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
				match (lhs_expr.eval(eval_var)?, rhs_expr.eval(eval_var)?)
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
								None => Err(Message::error_span("division by 0", &op_span.join(&rhs_expr.span())))
							},
							
							BinaryOp::Mod => match bigint_checked_rem(lhs, rhs)
							{
								Some(x) => Ok(ExpressionValue::Integer(x)),
								None => Err(Message::error_span("modulo by 0", &op_span.join(&rhs_expr.span())))
							},
							
							BinaryOp::Shl => match bigint_shl(lhs, rhs)
							{
								Some(x) => Ok(ExpressionValue::Integer(x)),
								None => Err(Message::error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
							},
							
							BinaryOp::Shr => match bigint_shr(lhs, rhs)
							{
								Some(x) => Ok(ExpressionValue::Integer(x)),
								None => Err(Message::error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
							},
							
							BinaryOp::UShr => match bigint_ushr(lhs, rhs)
							{
								Some(x) => Ok(ExpressionValue::Integer(x)),
								None => Err(Message::error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
							},
							
							BinaryOp::And  => Ok(ExpressionValue::Integer(bigint_and(lhs, rhs))),
							BinaryOp::Or   => Ok(ExpressionValue::Integer(bigint_or (lhs, rhs))),
							BinaryOp::Xor  => Ok(ExpressionValue::Integer(bigint_xor(lhs, rhs))),
							BinaryOp::Eq   => Ok(ExpressionValue::Bool(lhs.abs() == rhs)),
							BinaryOp::Ne   => Ok(ExpressionValue::Bool(lhs.abs() != rhs)),
							BinaryOp::Lt   => Ok(ExpressionValue::Bool(lhs.abs() <  rhs)),
							BinaryOp::Le   => Ok(ExpressionValue::Bool(lhs.abs() <= rhs)),
							BinaryOp::Gt   => Ok(ExpressionValue::Bool(lhs.abs() >  rhs)),
							BinaryOp::Ge   => Ok(ExpressionValue::Bool(lhs.abs() >= rhs)),
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
			
			&Expression::BitSlice(_, ref op_span, _, _, ref inner) =>
			{
				inner.eval(eval_var)
			}
		}
	}
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
	rhs.to_usize().map(|rhs| lhs >> rhs)
}


fn bigint_ushr(lhs: BigInt, rhs: BigInt) -> Option<BigInt>
{
	rhs.to_usize().map(|rhs| lhs.abs() >> rhs)
}


fn bigint_not(x: BigInt) -> BigInt
{
	unimplemented!()
}


fn bigint_and(lhs: BigInt, rhs: BigInt) -> BigInt
{
	unimplemented!()
}


fn bigint_or(lhs: BigInt, rhs: BigInt) -> BigInt
{
	unimplemented!()
}


fn bigint_xor(lhs: BigInt, rhs: BigInt) -> BigInt
{
	unimplemented!()
}