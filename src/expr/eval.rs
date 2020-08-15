use crate::*;
use std::collections::HashMap;


pub struct EvalContext
{
	locals: HashMap<String, expr::Value>
}


impl EvalContext
{
	pub fn new() -> EvalContext
	{
		EvalContext
		{
			locals: HashMap::new()
		}
	}
	
	
	pub fn set_local<S>(&mut self, name: S, value: expr::Value)
	where S: Into<String>
	{
		self.locals.insert(name.into(), value);
	}
	
	
	pub fn get_local(&self, name: &str) -> Result<expr::Value, ()>
	{
		match self.locals.get(name)
		{
			Some(value) => Ok(value.clone()),
			None => Err(())
		}
	}
}


pub struct EvalVariableInfo<'a>
{
	pub report: diagn::RcReport,
	pub hierarchy_level: usize,
	pub hierarchy: &'a Vec<String>,
	pub span: &'a diagn::Span,
}


pub struct EvalFunctionInfo<'a>
{
	pub report: diagn::RcReport,
	pub fn_index: usize,
	pub args: Vec<expr::Value>,
	pub span: &'a diagn::Span,
}


impl expr::Expr
{
	pub fn eval<FVar, FFn>(
		&self,
		report: RcReport,
		ctx: &mut EvalContext,
		eval_var: &FVar,
		eval_fn: &FFn)
		-> Result<expr::Value, ()>
	where
		FVar: Fn(&EvalVariableInfo) -> Result<expr::Value, bool>,
		FFn: Fn(&EvalFunctionInfo) -> Result<expr::Value, bool>
	{
		match self
		{
			&expr::Expr::Literal(_, ref value) => Ok(value.clone()),
			
			&expr::Expr::Variable(ref span, hierarchy_level, ref hierarchy) =>
			{
				if hierarchy_level == 0 && hierarchy.len() == 1
				{
					match ctx.get_local(&hierarchy[0])
					{
						Ok(value) => return Ok(value),
						Err(()) => {}
					}
				}

				let info = EvalVariableInfo
				{
					report: report.clone(),
					hierarchy_level,
					hierarchy,
					span,
				};

				match eval_var(&info)
				{
					Ok(value) => Ok(value),
					Err(handled) =>
					{
						if !handled
							{ report.error_span("unknown variable", &span); }
							
						Err(())
					}
				}
			}
			
			&expr::Expr::UnaryOp(ref span, _, op, ref inner_expr) =>
			{
				match inner_expr.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					expr::Value::Integer(ref x) => match op
					{
						expr::UnaryOp::Neg => Ok(expr::Value::make_integer(-x)),
						expr::UnaryOp::Not => Ok(expr::Value::make_integer(!x))
					},
					
					expr::Value::Bool(b) => match op
					{
						expr::UnaryOp::Not => Ok(expr::Value::Bool(!b)),
						_ => Err(report.error_span("invalid argument type to operator", &span))
					},
					
					_ => Err(report.error_span("invalid argument type to operator", &span))
				}
			}
			
			&expr::Expr::BinaryOp(ref span, ref op_span, op, ref lhs_expr, ref rhs_expr) =>
			{
				if op == expr::BinaryOp::Assign
				{
					use std::ops::Deref;
					
					match lhs_expr.deref()
					{
						&expr::Expr::Variable(_, hierarchy_level, ref hierarchy) =>
						{
							if hierarchy_level == 0 && hierarchy.len() == 1
							{
								match ctx.get_local(&hierarchy[0])
								{
									Ok(_) =>
									{
										let value = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
										ctx.set_local(hierarchy[0].clone(), value);
										return Ok(expr::Value::Void);
									}
									Err(()) => {}
								}
							}
							
							Err(report.error_span("symbol cannot be assigned to", &lhs_expr.span()))
						}
						
						_ => Err(report.error_span("invalid assignment destination", &lhs_expr.span()))
					}
				}
				
				else if op == expr::BinaryOp::LazyOr || op == expr::BinaryOp::LazyAnd
				{
					let lhs = lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
					
					match (op, &lhs)
					{
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(true))  => return Ok(lhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(false)) => return Ok(lhs),
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(false)) => { }
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(true))  => { }
						_ => return Err(report.error_span("invalid argument type to operator", &lhs_expr.span()))
					}
					
					let rhs = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
					
					match (op, &rhs)
					{
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(true))  => Ok(rhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(false)) => Ok(rhs),
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(false)) => Ok(rhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(true))  => Ok(rhs),
						_ => Err(report.error_span("invalid argument type to operator", &rhs_expr.span()))
					}
				}
				
				else
				{
					match (lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?, rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?)
					{
						(expr::Value::Integer(ref lhs), expr::Value::Integer(ref rhs)) =>
						{
							match op
							{
								expr::BinaryOp::Add => Ok(expr::Value::make_integer(lhs + rhs)),
								expr::BinaryOp::Sub => Ok(expr::Value::make_integer(lhs - rhs)),
								expr::BinaryOp::Mul => Ok(expr::Value::make_integer(lhs * rhs)),
								
								expr::BinaryOp::Div => match lhs.checked_div(rhs)
								{
									Some(x) => Ok(expr::Value::make_integer(x)),
									None => Err(report.error_span("division by zero", &op_span.join(&rhs_expr.span())))
								},
								
								expr::BinaryOp::Mod => match lhs.checked_rem(rhs)
								{
									Some(x) => Ok(expr::Value::make_integer(x)),
									None => Err(report.error_span("modulo by zero", &op_span.join(&rhs_expr.span())))
								},
								
								expr::BinaryOp::Shl => match lhs.checked_shl(rhs)
								{
									Some(x) => Ok(expr::Value::make_integer(x)),
									None => Err(report.error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
								},
								
								expr::BinaryOp::Shr => match lhs.checked_shr(rhs)
								{
									Some(x) => Ok(expr::Value::make_integer(x)),
									None => Err(report.error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
								},
								
								expr::BinaryOp::And  => Ok(expr::Value::make_integer(lhs & rhs)),
								expr::BinaryOp::Or   => Ok(expr::Value::make_integer(lhs | rhs)),
								expr::BinaryOp::Xor  => Ok(expr::Value::make_integer(lhs ^ rhs)),
								expr::BinaryOp::Eq   => Ok(expr::Value::Bool(lhs == rhs)),
								expr::BinaryOp::Ne   => Ok(expr::Value::Bool(lhs != rhs)),
								expr::BinaryOp::Lt   => Ok(expr::Value::Bool(lhs <  rhs)),
								expr::BinaryOp::Le   => Ok(expr::Value::Bool(lhs <= rhs)),
								expr::BinaryOp::Gt   => Ok(expr::Value::Bool(lhs >  rhs)),
								expr::BinaryOp::Ge   => Ok(expr::Value::Bool(lhs >= rhs)),
								
								expr::BinaryOp::Concat =>
								{
									// FIXME: wrongly evaluates lhs and rhs again, with possible duplicated side-effects
									//let lhs_sliced = lhs_expr.eval_slice(report.clone(), ctx, eval_var, eval_fn)?;
									//let rhs_sliced = rhs_expr.eval_slice(report.clone(), ctx, eval_var, eval_fn)?;

									match (lhs.size, rhs.size)
									{
										(Some(lhs_width), Some(rhs_width)) => Ok(expr::Value::make_integer(lhs.concat((lhs_width - 1, 0), &rhs, (rhs_width - 1, 0)))),
										(None, _) => Err(report.error_span("argument to concatenation with no known width", &lhs_expr.span())),
										(_, None) => Err(report.error_span("argument to concatenation with no known width", &rhs_expr.span()))
									}
								}

								_ => Err(report.error_span("invalid argument types to operator", &span))
							}
						}
						
						(expr::Value::Bool(lhs), expr::Value::Bool(rhs)) =>
						{
							match op
							{
								expr::BinaryOp::And => Ok(expr::Value::Bool(lhs & rhs)),
								expr::BinaryOp::Or  => Ok(expr::Value::Bool(lhs | rhs)),
								expr::BinaryOp::Xor => Ok(expr::Value::Bool(lhs ^ rhs)),
								expr::BinaryOp::Eq  => Ok(expr::Value::Bool(lhs == rhs)),
								expr::BinaryOp::Ne  => Ok(expr::Value::Bool(lhs != rhs)),
								_ => Err(report.error_span("invalid argument types to operator", &span))
							}
						}
						
						_ => Err(report.error_span("invalid argument types to operator", &span))
					}
				}
			}
			
			&expr::Expr::TernaryOp(_, ref cond, ref true_branch, ref false_branch) =>
			{
				match cond.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					expr::Value::Bool(true)  => true_branch.eval(report.clone(), ctx, eval_var, eval_fn),
					expr::Value::Bool(false) => false_branch.eval(report.clone(), ctx, eval_var, eval_fn),
					_ => Err(report.error_span("invalid condition type", &cond.span()))
				}
			}
			
			&expr::Expr::BitSlice(ref span, _, left, right, ref inner) =>
			{
				match inner.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					expr::Value::Integer(ref x) => Ok(expr::Value::make_integer(x.slice(left, right))),
					_ => Err(report.error_span("invalid argument type to slice", &span))
				}
			}
			
			&expr::Expr::SoftSlice(_, _, _, _, ref inner) =>
			{
				inner.eval(report, ctx, eval_var, eval_fn)
			}
			
			&expr::Expr::Block(_, ref exprs) =>
			{
				let mut result = expr::Value::Void;
				
				for expr in exprs
					{ result = expr.eval(report.clone(), ctx, eval_var, eval_fn)?; }
					
				Ok(result)
			}
			
			&expr::Expr::Call(ref span, ref target, ref arg_exprs) =>
			{
				match target.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					expr::Value::Function(id) =>
					{
						let mut args = Vec::new();
						for expr in arg_exprs
							{ args.push(expr.eval(report.clone(), ctx, eval_var, eval_fn)?); }

						let info = EvalFunctionInfo
						{
							report: report.clone(),
							fn_index: id,
							args,
							span,
						};
							
						match eval_fn(&info)
						{
							Ok(value) => Ok(value),
							Err(_handled) => Err(())
						}
					}
					
					_ => Err(report.error_span("expression is not callable", &target.span()))
				}
			}
		}
	}


	/*pub fn eval_slice<FVar, FFn>(&self, report: RcReport, ctx: &mut ExpressionEvalContext, eval_var: &FVar, eval_fn: &FFn) -> Result<expr::Value, ()>
		where
		FVar: Fn(RcReport, &str, &Span) -> Result<expr::Value, bool>,
		FFn: Fn(RcReport, usize, Vec<expr::Value>, &Span) -> Result<expr::Value, bool>
	{
		match self
		{
			&expr::Expr::SoftSlice(ref span, _, left, right, ref inner) =>
			{
				match inner.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					expr::Value::Integer(ref x) => Ok(expr::Value::make_integer(x.slice(left, right))),
					_ => Err(report.error_span("invalid argument type to slice", &span))
				}
			}

			_ => self.eval(report, ctx, eval_var, eval_fn)
		}
	}*/
}


impl expr::Value
{
	pub fn min_size(&self) -> usize
	{
		match &self
		{
			&expr::Value::Integer(bigint) => bigint.min_size(),
			_ => panic!("not an integer")
		}
	}
	

	pub fn get_bit(&self, index: usize) -> bool
	{
		match self
		{
			&expr::Value::Integer(ref bigint) => bigint.get_bit(index),
			_ => panic!("not an integer")
		}
	}
}