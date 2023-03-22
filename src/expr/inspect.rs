use crate::*;


pub struct StaticSizeInfo
{
	pub locals: std::collections::HashMap<String, usize>,
}


impl StaticSizeInfo
{
	pub fn new() -> StaticSizeInfo
	{
		StaticSizeInfo {
			locals: std::collections::HashMap::new(),
		}
	}
}


impl expr::Expr
{
	pub fn get_static_size(
		&self,
		info: &StaticSizeInfo)
		-> Option<usize>
	{
		match self
		{
			expr::Expr::Variable(_, hierarchy_level, ref hierarchy) =>
			{
				if *hierarchy_level != 0 ||
					hierarchy.len() != 1
				{
					return None;
				}

				if let Some(size) = info.locals.get(&hierarchy[0])
				{
					return Some(*size);
				}

				None
			}
			
			expr::Expr::Literal(_, expr::Value::Integer(util::BigInt { size: Some(size), .. })) =>
				Some(*size),
			
			expr::Expr::BinaryOp(_, _, expr::BinaryOp::Concat, ref lhs, ref rhs) =>
			{
				let lhs_size = lhs.get_static_size(info)?;
				let rhs_size = rhs.get_static_size(info)?;

				Some(lhs_size + rhs_size)
			}
			
			expr::Expr::BitSlice(_, _, left, right, _) =>
				Some(left - right),

			expr::Expr::SoftSlice(_, _, left, right, _) =>
				Some(left - right),
			
			expr::Expr::TernaryOp(_, _, ref true_branch, ref false_branch) =>
			{
				let true_size = true_branch.get_static_size(info)?;
				let false_size = false_branch.get_static_size(info)?;
				
				if true_size == false_size
				{
					Some(true_size)
				}
				else
				{
					None
				}
			}
			
			expr::Expr::Block(_, ref exprs) =>
				exprs.last()?.get_static_size(info),

			expr::Expr::Call(_, func, args) =>
			{
				if let expr::Expr::Literal(
					_,
					expr::Value::BuiltInFunction(ref builtin_name)) = *func.as_ref()
				{
					expr::get_static_size_builtin(builtin_name, info, &args)
				}
				else if let expr::Expr::Variable(_, 0, ref names) = *func.as_ref()
				{
					if names.len() == 1
					{
						expr::get_static_size_builtin(&names[0], info, &args)
					}
					else
					{
						None
					}
				}
				else
				{
					None
				}
			}
			
			_ => None
		}
	}
	
	
	pub fn returned_value_span(&self) -> &diagn::Span
	{
		match self
		{
			&expr::Expr::Block(ref span, ref exprs) =>
			{
				match exprs.last()
				{
					None => &span,
					Some(expr) => expr.returned_value_span()
				}
			}
			
			_ => self.span()
		}
	}
}