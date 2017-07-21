use diagn::{Span, Message};
use super::Expression;
use super::ExpressionValue;
use super::ExpressionType;
use super::UnaryOp;
use super::BinaryOp;


impl Expression
{
	pub fn width(&self) -> Option<usize>
	{
		match self
		{
			&Expression::BitSlice(_, _, left, right, _) => Some(left + 1 - right),
			_ => None
		}
	}


	pub fn check_vars<F>(&self, check_var: &F) -> Result<(), Message>
	where F: Fn(&str, &Span) -> Result<(), Message>
	{
		match self
		{
			&Expression::Literal(..) => Ok(()),
			
			&Expression::Variable(ref span, ref name) => check_var(&name, &span),
			
			&Expression::UnaryOp(.., ref inner) => inner.check_vars(check_var),
			
			&Expression::BinaryOp(.., ref lhs, ref rhs) =>
			{
				lhs.check_vars(check_var)?;
				rhs.check_vars(check_var)
			}
			
			&Expression::BitSlice(.., ref inner) => inner.check_vars(check_var)
		}
	}


	pub fn eval_type<F>(&self, get_var_type: &F) -> Result<ExpressionType, Message>
	where F: Fn(&str) -> ExpressionType
	{
		match self
		{
			&Expression::Literal(_, ref value) => match value
			{
				&ExpressionValue::Integer(_) => Ok(ExpressionType::Integer),
				&ExpressionValue::Bool(_) => Ok(ExpressionType::Bool)
			},
			
			&Expression::Variable(_, ref name) => Ok(get_var_type(&name)),
			
			&Expression::UnaryOp(_, ref op_span, op, ref inner) =>
			{
				let inner_type = inner.eval_type(get_var_type)?;
				
				match op
				{
					UnaryOp::Neg => ensure_unary_int_to_int(inner_type, &op_span),
					UnaryOp::Not => ensure_unary_any_to_same(inner_type)
				}
			}
			
			&Expression::BinaryOp(_, ref op_span, op, ref lhs, ref rhs) =>
			{
				let lhs_type = lhs.eval_type(get_var_type)?;
				let rhs_type = rhs.eval_type(get_var_type)?;
				
				match op
				{
					BinaryOp::Add |
					BinaryOp::Sub |
					BinaryOp::Mul |
					BinaryOp::Div |
					BinaryOp::Mod |
					BinaryOp::Shl |
					BinaryOp::Shr |
					BinaryOp::UShr => ensure_binary_int_to_int(lhs_type, rhs_type, &op_span),
					
					BinaryOp::And |
					BinaryOp::Or |
					BinaryOp::Xor => ensure_binary_any_to_same(lhs_type, rhs_type, &op_span),
					
					BinaryOp::Eq |
					BinaryOp::Ne |
					BinaryOp::Lt |
					BinaryOp::Le |
					BinaryOp::Gt |
					BinaryOp::Ge => ensure_binary_any_to_bool(lhs_type, rhs_type, &op_span),
					
					BinaryOp::LazyAnd |
					BinaryOp::LazyOr => ensure_binary_bool_to_bool(lhs_type, rhs_type, &op_span)
				}
			}
			
			&Expression::BitSlice(_, ref op_span, _, _, ref inner) =>
			{
				let inner_type = inner.eval_type(get_var_type)?;
				
				if inner_type == ExpressionType::Integer
					{ Ok(ExpressionType::Integer) }
				else
					{ Err(Message::error_span("expected integer argument to bit slice", &op_span)) }
			}
		}
	}
}


fn ensure_unary_any_to_same(inner_type: ExpressionType) -> Result<ExpressionType, Message>
{
	Ok(inner_type)
}


fn ensure_unary_int_to_int(inner_type: ExpressionType, span: &Span) -> Result<ExpressionType, Message>
{
	if inner_type == ExpressionType::Integer
		{ Ok(ExpressionType::Integer) }
	else
		{ Err(Message::error_span("expected integer argument to unary op", span)) }
}


fn ensure_binary_int_to_int(lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, Message>
{
	if lhs_type == ExpressionType::Integer && rhs_type == ExpressionType::Integer
		{ Ok(ExpressionType::Integer) }
	else
		{ Err(Message::error_span("expected integer arguments to binary op", span)) }
}


fn ensure_binary_bool_to_bool(lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, Message>
{
	if lhs_type == ExpressionType::Bool && rhs_type == ExpressionType::Bool
		{ Ok(ExpressionType::Bool) }
	else
		{ Err(Message::error_span("expected bool arguments to binary op", span)) }
}


fn ensure_binary_any_to_bool(lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, Message>
{
	if lhs_type == rhs_type
		{ Ok(ExpressionType::Bool) }
	else
		{ Err(Message::error_span("expected arguments of the same type to binary op", span)) }
}


fn ensure_binary_any_to_same(lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, Message>
{
	if lhs_type == rhs_type
		{ Ok(lhs_type) }
	else
		{ Err(Message::error_span("expected arguments of the same type to binary op", span)) }
}