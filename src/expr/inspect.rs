use crate::*;


pub struct StaticallyKnownProvider<'a>
{
	pub locals: std::collections::HashMap<String, StaticallyKnownLocal>,
	pub query_nesting_level: &'a dyn Fn(&StaticallyKnownNestingLevelQuery) -> bool,
	pub query_variable: &'a dyn Fn(&StaticallyKnownVariableQuery) -> bool,
	pub query_member: &'a dyn Fn(&StaticallyKnownMemberQuery) -> bool,
	pub query_function: &'a dyn Fn(&StaticallyKnownFunctionQuery) -> bool,
}


pub struct StaticallyKnownNestingLevelQuery
{
	pub hierarchy_level: usize,
}


pub struct StaticallyKnownVariableQuery<'a>
{
	pub hierarchy_level: usize,
	pub hierarchy: &'a Vec<String>,
}


pub struct StaticallyKnownMemberQuery<'a>
{
	pub value: &'a expr::Expr,
	pub member_name: &'a String,
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
			query_nesting_level: &|_| false,
			query_variable: &|_| false,
			query_member: &|_| false,
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
			expr::Expr::Variable(_, name) =>
			{
				if let Some(StaticallyKnownLocal { size: Some(size), .. }) = provider.locals.get(name)
				{
					return Some(*size);
				}

				None
			}

			expr::Expr::NestingLevel { .. } => None,
			expr::Expr::MemberAccess { .. } => None,
			
			expr::Expr::Literal(_, expr::Value::Integer(_, util::BigInt { size: Some(size), .. })) =>
				Some(*size),

			expr::Expr::Literal(..) => None,

			expr::Expr::UnaryOp(..) => None,
			
			expr::Expr::BinaryOp(_, _, expr::BinaryOp::Concat, lhs, rhs) =>
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
			
			expr::Expr::TernaryOp(_, _, true_branch, false_branch) =>
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
			
			expr::Expr::Block(_, exprs) =>
				exprs.last()?.get_static_size(provider),

			expr::Expr::Call(_, func, args) =>
			{
				if let expr::Expr::Variable(_, ref name) = *func.as_ref()
				{
					expr::get_static_size_builtin_fn(
						name,
						provider,
						&args)
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
			expr::Expr::Variable(_, name) =>
			{
				if let Some(var) = provider.locals.get(name)
				{
					return var.value_known;
				}

				let query = StaticallyKnownVariableQuery {
					hierarchy: &vec![name.clone()],
					hierarchy_level: 0,
				};

				(provider.query_variable)(&query)
			}

			expr::Expr::NestingLevel { .. } => false,
			expr::Expr::MemberAccess { .. } => false,

			/* TODO
			
			&expr::Expr::NestingLevel { span, nesting_level } =>
			{
				let mut query = EvalCtxLabelQuery {
					report,
					span,
					nesting_level,
				};

				provider(EvalQuery::CtxLabel(&mut query))
			}	

			&expr::Expr::MemberAccess { span, ref lhs, ref member_name } =>
			{
				let value = propagate!(lhs
					.eval_with_ctx(report, ctx, provider)?);

				let mut query = EvalMemberQuery {
					report,
					span,
					value,
					member_name,
				};

				provider(EvalQuery::Member(&mut query))
			}*/
			
			expr::Expr::Literal(_, _) => true,

			expr::Expr::UnaryOp(..) => false,
			
			expr::Expr::BinaryOp(_, _, _, lhs, rhs) =>
			{
				let lhs_known = lhs.is_value_statically_known(provider);
				let rhs_known = rhs.is_value_statically_known(provider);

				lhs_known && rhs_known
			}
			
			expr::Expr::Slice(_, _, left_expr, right_expr, expr) =>
				left_expr.is_value_statically_known(provider) &&
				right_expr.is_value_statically_known(provider) &&
				expr.is_value_statically_known(provider),
			
			expr::Expr::SliceShort(_, _, size_expr, expr) =>
				size_expr.is_value_statically_known(provider) &&
				expr.is_value_statically_known(provider),
			
			expr::Expr::TernaryOp(_, condition, true_branch, false_branch) =>
			{
				let condition_known = condition.is_value_statically_known(provider);
				let true_known = true_branch.is_value_statically_known(provider);
				let false_known = false_branch.is_value_statically_known(provider);
				
				condition_known && true_known && false_known
			}
			
			expr::Expr::Block(_, exprs) =>
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
				if let expr::Expr::Variable(_, ref name) = *func.as_ref()
				{
					for arg in args
					{
						if !arg.is_value_statically_known(provider)
						{
							return false;
						}
					}
					
					if expr::get_statically_known_value_builtin_fn(
						name,
						&args)
					{
						return true;
					}
						
					let query = StaticallyKnownFunctionQuery {
						func: name,
						args,
					};

					(provider.query_function)(&query)
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