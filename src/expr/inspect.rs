use crate::diagn::Span;
use super::Expression;
use super::ExpressionValue;
use super::BinaryOp;


impl Expression
{
	pub fn width(&self) -> Option<usize>
	{
		if let Some(slice) = self.slice()
			{ Some(slice.0 + 1 - slice.1) }
		else
			{ None }
	}


	pub fn has_size(&self) -> bool
	{
		if let Some(_) = self.slice()
			{ true }
		else
			{ false }
	}
	
	
	pub fn slice(&self) -> Option<(usize, usize)>
	{
		match self
		{
			&Expression::Literal(_, ExpressionValue::Integer{ size: Some(size), .. }) =>
			{
				Some((size - 1, 0))
			}
			
			&Expression::BinaryOp(_, _, BinaryOp::Concat, ref lhs, ref rhs) =>
			{
				let lhs_width = lhs.width();
				let rhs_width = rhs.width();
				
				if lhs_width.is_none() || rhs_width.is_none()
					{ return None; }
					
				Some((lhs_width.unwrap() + rhs_width.unwrap() - 1, 0))
			}
			
			&Expression::BitSlice(_, _, left, right, _) => Some((left, right)),
			&Expression::SoftSlice(_, _, left, right, _) => Some((left, right)),
			
			&Expression::TernaryOp(_, _, ref true_branch, ref false_branch) =>
			{
				let true_width = true_branch.width();
				let false_width = false_branch.width();
				
				if true_width.is_none() || false_width.is_none()
					{ return None; }
					
				if true_width.unwrap() != false_width.unwrap()
					{ return None; }
					
				Some((true_width.unwrap() - 1, 0))
			}
			
			&Expression::Block(_, ref exprs) =>
			{
				match exprs.last()
				{
					None => None,
					Some(expr) => expr.slice()
				}
			}
			
			_ => None
		}
	}
	
	
	pub fn returned_value_span(&self) -> Span
	{
		match self
		{
			&Expression::Block(ref span, ref exprs) =>
			{
				match exprs.last()
				{
					None => span.clone(),
					Some(expr) => expr.returned_value_span()
				}
			}
			
			_ => self.span()
		}
	}
}