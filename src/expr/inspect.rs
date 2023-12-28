use crate::*;


pub struct StaticallyKnownProvider<'a>
{
	pub locals: std::collections::HashMap<String, StaticallyKnownLocal>,
	pub query_variable: &'a dyn Fn(&StaticallyKnownVariableQuery) -> bool,
	pub query_function: &'a dyn Fn(&StaticallyKnownFunctionQuery) -> bool,
}


pub struct StaticallyKnownVariableQuery<'a>
{
	pub hierarchy_level: usize,
	pub hierarchy: &'a Vec<String>,
}


pub struct StaticallyKnownFunctionQuery<'a>
{
	pub func: &'a str,
	pub args: &'a Vec<expr::Expr>,
}


impl<'a> StaticallyKnownProvider<'a>
{
	pub fn new() -> StaticallyKnownProvider<'a>
	{
		StaticallyKnownProvider {
			locals: std::collections::HashMap::new(),
			query_variable: &|_| false,
			query_function: &|_| false,
		}
	}
}


pub struct StaticallyKnownLocal
{
	pub size: Option<usize>,
	pub value_known: bool,
}


impl StaticallyKnownLocal
{
	pub fn new() -> StaticallyKnownLocal
	{
		StaticallyKnownLocal {
			size: None,
			value_known: false,
		}
	}
}


impl expr::Expr
{
	pub fn get_static_size(
		&self,
		provider: &StaticallyKnownProvider)
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

				if let Some(StaticallyKnownLocal { size: Some(size), .. }) =
					provider.locals.get(&hierarchy[0])
				{
					return Some(*size);
				}

				None
			}
			
			expr::Expr::Literal(_, expr::Value::Integer(util::BigInt { size: Some(size), .. })) =>
				Some(*size),

			expr::Expr::Literal(..) => None,

			expr::Expr::UnaryOp(..) => None,
			
			expr::Expr::BinaryOp(_, _, expr::BinaryOp::Concat, ref lhs, ref rhs) =>
			{
				let lhs_size = lhs.get_static_size(provider)?;
				let rhs_size = rhs.get_static_size(provider)?;

				Some(lhs_size + rhs_size)
			}

			expr::Expr::BinaryOp(..) => None,
			
			expr::Expr::Slice(_, _, left_expr, right_expr, _) =>
			{
				let left = left_expr.try_eval_usize()? + 1;
				let right = right_expr.try_eval_usize()?;

				if right > left
				{
					return None;
				}

				Some(left - right)
			}
			
			expr::Expr::SliceShort(_, _, size_expr, _) =>
			{
				let size = size_expr.try_eval_usize()?;

				Some(size)
			}
			
			expr::Expr::TernaryOp(_, _, ref true_branch, ref false_branch) =>
			{
				let true_size = true_branch.get_static_size(provider)?;
				let false_size = false_branch.get_static_size(provider)?;
				
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
				exprs.last()?.get_static_size(provider),

			expr::Expr::Call(_, func, args) =>
			{
				if let expr::Expr::Variable(_, 0, ref names) = *func.as_ref()
				{
					if names.len() == 1
					{
						expr::get_static_size_builtin_fn(
							&names[0],
							provider,
							&args)
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

			expr::Expr::Asm(..) => None,
		}
	}


	pub fn is_value_statically_known(
		&self,
		provider: &StaticallyKnownProvider)
		-> bool
	{
		match self
		{
			expr::Expr::Variable(_, hierarchy_level, ref hierarchy) =>
			{
				if *hierarchy_level == 0 && hierarchy.len() == 1
				{
					if let Some(var) = provider.locals.get(&hierarchy[0])
					{
						return var.value_known;
					}
				}

				let query = StaticallyKnownVariableQuery {
					hierarchy,
					hierarchy_level: *hierarchy_level,
				};

				(provider.query_variable)(&query)
			}
			
			expr::Expr::Literal(_, _) => true,

			expr::Expr::UnaryOp(..) => false,
			
			expr::Expr::BinaryOp(_, _, _, ref lhs, ref rhs) =>
			{
				let lhs_known = lhs.is_value_statically_known(provider);
				let rhs_known = rhs.is_value_statically_known(provider);

				lhs_known && rhs_known
			}
			
			expr::Expr::Slice(_, _, ref left_expr, ref right_expr, ref expr) =>
				left_expr.is_value_statically_known(provider) &&
				right_expr.is_value_statically_known(provider) &&
				expr.is_value_statically_known(provider),
			
			expr::Expr::SliceShort(_, _, ref size_expr, ref expr) =>
				size_expr.is_value_statically_known(provider) &&
				expr.is_value_statically_known(provider),
			
			expr::Expr::TernaryOp(_, ref condition, ref true_branch, ref false_branch) =>
			{
				let condition_known = condition.is_value_statically_known(provider);
				let true_known = true_branch.is_value_statically_known(provider);
				let false_known = false_branch.is_value_statically_known(provider);
				
				condition_known && true_known && false_known
			}
			
			expr::Expr::Block(_, ref exprs) =>
			{
				for expr in exprs
				{
					if !expr.is_value_statically_known(provider)
					{
						return false;
					}
				}
				
				true
			}

			expr::Expr::Call(_, func, args) =>
			{
				if let expr::Expr::Variable(_, 0, ref names) = *func.as_ref()
				{
					for arg in args
					{
						if !arg.is_value_statically_known(provider)
						{
							return false;
						}
					}
					
					if names.len() == 1
					{
						if expr::get_statically_known_value_builtin_fn(
							&names[0],
							&args)
						{
							return true;
						}
							
						let query = StaticallyKnownFunctionQuery {
							func: &names[0],
							args,
						};

						(provider.query_function)(&query)
					}
					else
					{
						false
					}
				}
				else
				{
					false
				}
			}

			expr::Expr::Asm(..) => false,
		}
	}
	
	
	pub fn returned_value_span(&self) -> diagn::Span
	{
		match self
		{
			&expr::Expr::Block(span, ref exprs) =>
			{
				match exprs.last()
				{
					None => span,
					Some(expr) => expr.returned_value_span()
				}
			}
			
			_ => self.span()
		}
	}
}