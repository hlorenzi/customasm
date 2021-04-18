use crate::*;


impl expr::Expr
{
	pub fn width(&self) -> Option<usize>
	{
		if let Some(slice) = self.slice()
			{ Some(slice.0 + 1 - slice.1) }
		else
			{ None }
	}


	pub fn size(&self) -> Option<usize>
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
			&expr::Expr::Literal(_, expr::Value::Integer(util::BigInt { size: Some(size), .. })) =>
			{
				Some((size, 0))
			}
			
			&expr::Expr::BinaryOp(_, _, expr::BinaryOp::Concat, ref lhs, ref rhs) =>
			{
				let lhs_width = lhs.width();
				let rhs_width = rhs.width();
				
				if lhs_width.is_none() || rhs_width.is_none()
					{ return None; }
					
				Some((lhs_width.unwrap() + rhs_width.unwrap(), 0))
			}
			
			&expr::Expr::BitSlice(_, _, left, right, _) => Some((left, right)),
			&expr::Expr::SoftSlice(_, _, left, right, _) => Some((left, right)),
			
			&expr::Expr::TernaryOp(_, _, ref true_branch, ref false_branch) =>
			{
				let true_width = true_branch.width();
				let false_width = false_branch.width();
				
				if true_width.is_none() || false_width.is_none()
					{ return None; }
					
				if true_width.unwrap() != false_width.unwrap()
					{ return None; }
					
				Some((true_width.unwrap(), 0))
			}
			
			&expr::Expr::Block(_, ref exprs) =>
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
	
	
	pub fn returned_value_span(&self) -> diagn::Span
	{
		match self
		{
			&expr::Expr::Block(ref span, ref exprs) =>
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