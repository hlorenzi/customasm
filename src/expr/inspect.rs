use super::Expression;
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
			
			&Expression::Block(_, ref exprs) =>
			{
				match exprs.last()
				{
					None => None,
					Some(expr) => expr.width()
				}
			}
			
			_ => None
		}
	}
	
	
	pub fn slice(&self) -> Option<(usize, usize)>
	{
		match self
		{
			&Expression::BinaryOp(_, _, BinaryOp::Concat, _, _) => self.width().map(|w| (w - 1, 0)),
			&Expression::BitSlice(_, _, left, right, _) => Some((left, right)),
			
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
}