use crate::*;


pub struct StaticallyKnownProvider<'a>
{
	pub opts: &'a asm::AssemblyOptions,
	pub locals: std::collections::HashMap<String, StaticallyKnownLocal>,
}


impl<'a> StaticallyKnownProvider<'a>
{
	pub fn new(opts: &'a asm::AssemblyOptions) -> StaticallyKnownProvider<'a>
	{
		StaticallyKnownProvider {
			opts,
			locals: std::collections::HashMap::new(),
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
	pub fn size_guess(
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

			expr::Expr::StructInit { .. } => None,
			expr::Expr::NestingLevel { .. } => None,
			expr::Expr::MemberAccess { .. } => None,
			
			expr::Expr::Literal(_, expr::Value::Integer(_, util::BigInt { size: Some(size), .. })) =>
				Some(*size),

			expr::Expr::Literal(..) => None,

			expr::Expr::UnaryOp(..) => None,
			
			expr::Expr::BinaryOp(_, _, expr::BinaryOp::Concat, lhs, rhs) =>
			{
				let lhs_size = lhs.size_guess(provider)?;
				let rhs_size = rhs.size_guess(provider)?;

				Some(lhs_size + rhs_size)
			}

			expr::Expr::BinaryOp(..) => None,
			
			expr::Expr::Slice(_, _, left_expr, right_expr, _) =>
			{
				let left = left_expr.try_eval_usize(provider.opts)? + 1;
				let right = right_expr.try_eval_usize(provider.opts)?;

				if right > left
				{
					return None;
				}

				Some(left - right)
			}
			
			expr::Expr::SliceShort(_, _, size_expr, _) =>
			{
				let size = size_expr.try_eval_usize(provider.opts)?;

				Some(size)
			}
			
			expr::Expr::TernaryOp(_, _, true_branch, false_branch) =>
			{
				let true_size = true_branch.size_guess(provider)?;
				let false_size = false_branch.size_guess(provider)?;
				
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
				exprs.last()?.size_guess(provider),

			expr::Expr::Call(_, func, args) =>
			{
				if let expr::Expr::Variable(_, ref name) = *func.as_ref()
				{
					if let Some(builtin_fn) = expr::resolve_builtin_fn(name, provider.opts)
					{
						expr::get_builtin_fn_size_guess(
							builtin_fn,
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
}