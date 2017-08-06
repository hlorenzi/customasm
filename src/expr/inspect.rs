use diagn::{Span, Report};
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
			&Expression::BinaryOp(_, _, BinaryOp::Concat, ref lhs, ref rhs) =>
			{
				let lhs_width = lhs.width();
				let rhs_width = rhs.width();
				
				if lhs_width.is_none() || rhs_width.is_none()
					{ return None; }
					
				Some(lhs_width.unwrap() + rhs_width.unwrap())
			}
			
			&Expression::BitSlice(_, _, left, right, _) => Some(left + 1 - right),
			_ => None
		}
	}
	
	
	pub fn slice(&self) -> Option<(usize, usize)>
	{
		match self
		{
			&Expression::BinaryOp(_, _, BinaryOp::Concat, _, _) => self.width().map(|w| (w - 1, 0)),
			&Expression::BitSlice(_, _, left, right, _) => Some((left, right)),
			_ => None
		}
	}


	pub fn check_vars<F>(&self, check_var: &mut F) -> Result<(), ()>
	where F: FnMut(&str, &Span) -> Result<(), ()>
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


	pub fn eval_type<F>(&self, report: &mut Report, get_var_type: &F) -> Result<ExpressionType, ()>
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
				let inner_type = inner.eval_type(report, get_var_type)?;
				
				match op
				{
					UnaryOp::Neg => ensure_unary_int_to_int(report, inner_type, &op_span),
					UnaryOp::Not => ensure_unary_any_to_same(inner_type)
				}
			}
			
			&Expression::BinaryOp(_, ref op_span, op, ref lhs, ref rhs) =>
			{
				let lhs_type = lhs.eval_type(report, get_var_type)?;
				let rhs_type = rhs.eval_type(report, get_var_type)?;
				
				match op
				{
					BinaryOp::Add |
					BinaryOp::Sub |
					BinaryOp::Mul |
					BinaryOp::Div |
					BinaryOp::Mod |
					BinaryOp::Shl |
					BinaryOp::Shr |
					BinaryOp::Concat => ensure_binary_int_to_int(report, lhs_type, rhs_type, &op_span),
					
					BinaryOp::And |
					BinaryOp::Or |
					BinaryOp::Xor => ensure_binary_any_to_same(report, lhs_type, rhs_type, &op_span),
					
					BinaryOp::Eq |
					BinaryOp::Ne |
					BinaryOp::Lt |
					BinaryOp::Le |
					BinaryOp::Gt |
					BinaryOp::Ge => ensure_binary_any_to_bool(report, lhs_type, rhs_type, &op_span),
					
					BinaryOp::LazyAnd |
					BinaryOp::LazyOr => ensure_binary_bool_to_bool(report, lhs_type, rhs_type, &op_span)
				}
			}
			
			&Expression::BitSlice(_, ref op_span, _, _, ref inner) =>
			{
				let inner_type = inner.eval_type(report, get_var_type)?;
				
				if inner_type == ExpressionType::Integer
					{ Ok(ExpressionType::Integer) }
				else
					{ Err(report.error_span("expected integer argument to bit slice", &op_span)) }
			}
		}
	}
}


fn ensure_unary_any_to_same(inner_type: ExpressionType) -> Result<ExpressionType, ()>
{
	Ok(inner_type)
}


fn ensure_unary_int_to_int(report: &mut Report, inner_type: ExpressionType, span: &Span) -> Result<ExpressionType, ()>
{
	if inner_type == ExpressionType::Integer
		{ Ok(ExpressionType::Integer) }
	else
		{ Err(report.error_span("expected integer argument to unary op", span)) }
}


fn ensure_binary_int_to_int(report: &mut Report, lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, ()>
{
	if lhs_type == ExpressionType::Integer && rhs_type == ExpressionType::Integer
		{ Ok(ExpressionType::Integer) }
	else
		{ Err(report.error_span("expected integer arguments to binary op", span)) }
}


fn ensure_binary_bool_to_bool(report: &mut Report, lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, ()>
{
	if lhs_type == ExpressionType::Bool && rhs_type == ExpressionType::Bool
		{ Ok(ExpressionType::Bool) }
	else
		{ Err(report.error_span("expected bool arguments to binary op", span)) }
}


fn ensure_binary_any_to_bool(report: &mut Report, lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, ()>
{
	if lhs_type == rhs_type
		{ Ok(ExpressionType::Bool) }
	else
		{ Err(report.error_span("expected arguments of the same type to binary op", span)) }
}


fn ensure_binary_any_to_same(report: &mut Report, lhs_type: ExpressionType, rhs_type: ExpressionType, span: &Span) -> Result<ExpressionType, ()>
{
	if lhs_type == rhs_type
		{ Ok(lhs_type) }
	else
		{ Err(report.error_span("expected arguments of the same type to binary op", span)) }
}