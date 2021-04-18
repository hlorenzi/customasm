use crate::*;
use std::collections::HashMap;


pub struct EvalContext
{
	locals: HashMap<String, expr::Value>,
	token_subs: HashMap<String, Vec<syntax::Token>>,
}


impl EvalContext
{
	pub fn new() -> EvalContext
	{
		EvalContext
		{
			locals: HashMap::new(),
			token_subs: HashMap::new(),
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
	
	
	pub fn set_token_sub<S>(&mut self, name: S, tokens: Vec<syntax::Token>)
	where S: Into<String>
	{
		self.token_subs.insert(name.into(), tokens);
	}
	
	
	pub fn get_token_sub<'a>(&'a self, name: &str) -> Option<&'a Vec<syntax::Token>>
	{
		self.token_subs.get(name)
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
	pub func: expr::Value,
	pub args: Vec<expr::Value>,
	pub span: &'a diagn::Span,
}


pub struct EvalAsmInfo<'a>
{
	pub report: diagn::RcReport,
	pub tokens: &'a [syntax::Token],
	pub span: &'a diagn::Span,
	pub args: &'a mut EvalContext,
}


impl expr::Expr
{
	pub fn eval<FVar, FFn, FAsm>(
		&self,
		report: diagn::RcReport,
		ctx: &mut EvalContext,
		eval_var: &FVar,
		eval_fn: &FFn,
		eval_asm: &FAsm)
		-> Result<expr::Value, ()>
	where
		FVar: Fn(&EvalVariableInfo) -> Result<expr::Value, bool>,
		FFn: Fn(&EvalFunctionInfo) -> Result<expr::Value, bool>,
		FAsm: Fn(&mut EvalAsmInfo) -> Result<expr::Value, ()>
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
				match inner_expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?
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
								let value = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?;
								ctx.set_local(hierarchy[0].clone(), value);
								return Ok(expr::Value::Void);
							}
							
							Err(report.error_span("symbol cannot be assigned to", &lhs_expr.span()))
						}
						
						_ => Err(report.error_span("invalid assignment destination", &lhs_expr.span()))
					}
				}
				
				else if op == expr::BinaryOp::LazyOr || op == expr::BinaryOp::LazyAnd
				{
					let lhs = lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?;
					
					match (op, &lhs)
					{
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(true))  => return Ok(lhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(false)) => return Ok(lhs),
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(false)) => { }
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(true))  => { }
						_ => return Err(report.error_span("invalid argument type to operator", &lhs_expr.span()))
					}
					
					let rhs = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?;
					
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
					match (lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?, rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?)
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
										(Some(lhs_width), Some(rhs_width)) => Ok(expr::Value::make_integer(lhs.concat((lhs_width, 0), &rhs, (rhs_width, 0)))),
										(None, _) => Err(report.error_span("argument to concatenation with unspecified size", &lhs_expr.span())),
										(_, None) => Err(report.error_span("argument to concatenation with unspecified size", &rhs_expr.span()))
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
				match cond.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?
				{
					expr::Value::Bool(true)  => true_branch.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm),
					expr::Value::Bool(false) => false_branch.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm),
					_ => Err(report.error_span("invalid condition type", &cond.span()))
				}
			}
			
			&expr::Expr::BitSlice(ref span, _, left, right, ref inner) =>
			{
				match inner.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?
				{
					expr::Value::Integer(ref x) => Ok(expr::Value::make_integer(x.slice(left, right))),
					_ => Err(report.error_span("invalid argument type to slice", &span))
				}
			}
			
			&expr::Expr::SoftSlice(_, _, _, _, ref inner) =>
			{
				inner.eval(report, ctx, eval_var, eval_fn, eval_asm)
			}
			
			&expr::Expr::Block(_, ref exprs) =>
			{
				let mut result = expr::Value::Void;
				
				for expr in exprs
					{ result = expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?; }
					
				Ok(result)
			}
			
			&expr::Expr::Call(ref span, ref target, ref arg_exprs) =>
			{
				let func = target.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?;

				match func
				{
					expr::Value::Function(_) =>
					{
						let mut args = Vec::new();
						for expr in arg_exprs
							{ args.push(expr.eval(report.clone(), ctx, eval_var, eval_fn, eval_asm)?); }

						let info = EvalFunctionInfo
						{
							report: report.clone(),
							func,
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
			
			&expr::Expr::Asm(ref span, ref tokens) =>
			{
				let mut info = EvalAsmInfo
				{
					report: report.clone(),
					tokens,
					span,
					args: ctx,
				};

				match eval_asm(&mut info)
				{
					Ok(value) => Ok(value),
					Err(_) => Err(())
				}
			}
		}
	}
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