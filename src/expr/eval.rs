use crate::diagn::{RcReport, Span};
use super::Expression;
use super::ExpressionValue;
use super::UnaryOp;
use super::BinaryOp;
use std::collections::HashMap;


pub struct ExpressionEvalContext
{
	locals: HashMap<String, ExpressionValue>
}


impl ExpressionEvalContext
{
	pub fn new() -> ExpressionEvalContext
	{
		ExpressionEvalContext
		{
			locals: HashMap::new()
		}
	}
	
	
	pub fn set_local<S>(&mut self, name: S, value: ExpressionValue)
	where S: Into<String>
	{
		self.locals.insert(name.into(), value);
	}
	
	
	pub fn get_local(&self, name: &str) -> Result<ExpressionValue, ()>
	{
		match self.locals.get(name)
		{
			Some(value) => Ok(value.clone()),
			None => Err(())
		}
	}
}


impl Expression
{
	pub fn eval<FVar, FFn>(&self, report: RcReport, ctx: &mut ExpressionEvalContext, eval_var: &FVar, eval_fn: &FFn) -> Result<ExpressionValue, ()>
		where
		FVar: Fn(RcReport, &str, &Span) -> Result<ExpressionValue, bool>,
		FFn: Fn(RcReport, usize, Vec<ExpressionValue>, &Span) -> Result<ExpressionValue, bool>
	{
		match self
		{
			&Expression::Literal(_, ref value) => Ok(value.clone()),
			
			&Expression::Variable(ref span, ref name) => match ctx.get_local(&name)
			{
				Ok(value) => Ok(value),
				Err(_) => match eval_var(report.clone(), &name, &span)
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
			
			&Expression::UnaryOp(ref span, _, op, ref inner_expr) =>
			{
				match inner_expr.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Integer(ref x) => match op
					{
						UnaryOp::Neg => Ok(ExpressionValue::make_integer(-x)),
						UnaryOp::Not => Ok(ExpressionValue::make_integer(!x))
					},
					
					ExpressionValue::Bool(b) => match op
					{
						UnaryOp::Not => Ok(ExpressionValue::Bool(!b)),
						_ => Err(report.error_span("invalid argument type to operator", &span))
					},
					
					_ => Err(report.error_span("invalid argument type to operator", &span))
				}
			}
			
			&Expression::BinaryOp(ref span, ref op_span, op, ref lhs_expr, ref rhs_expr) =>
			{
				if op == BinaryOp::Assign
				{
					use std::ops::Deref;
					
					match lhs_expr.deref()
					{
						&Expression::Variable(_, ref name) =>
						{
							let value = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
							ctx.set_local(name.clone(), value);
							Ok(ExpressionValue::Void)
						}
						
						_ => Err(report.error_span("invalid assignment destination", &lhs_expr.span()))
					}
				}
				
				else if op == BinaryOp::LazyOr || op == BinaryOp::LazyAnd
				{
					let lhs = lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
					
					match (op, &lhs)
					{
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(true))  => return Ok(lhs),
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(false)) => return Ok(lhs),
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(false)) => { }
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(true))  => { }
						_ => return Err(report.error_span("invalid argument type to operator", &lhs_expr.span()))
					}
					
					let rhs = rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?;
					
					match (op, &rhs)
					{
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(true))  => Ok(rhs),
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(false)) => Ok(rhs),
						(BinaryOp::LazyOr,  &ExpressionValue::Bool(false)) => Ok(rhs),
						(BinaryOp::LazyAnd, &ExpressionValue::Bool(true))  => Ok(rhs),
						_ => Err(report.error_span("invalid argument type to operator", &rhs_expr.span()))
					}
				}
				
				else
				{
					match (lhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?, rhs_expr.eval(report.clone(), ctx, eval_var, eval_fn)?)
					{
						(ExpressionValue::Integer(ref lhs), ExpressionValue::Integer(ref rhs)) =>
						{
							match op
							{
								BinaryOp::Add => Ok(ExpressionValue::make_integer(lhs + rhs)),
								BinaryOp::Sub => Ok(ExpressionValue::make_integer(lhs - rhs)),
								BinaryOp::Mul => Ok(ExpressionValue::make_integer(lhs * rhs)),
								
								BinaryOp::Div => match lhs.checked_div(rhs)
								{
									Some(x) => Ok(ExpressionValue::make_integer(x)),
									None => Err(report.error_span("division by zero", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::Mod => match lhs.checked_rem(rhs)
								{
									Some(x) => Ok(ExpressionValue::make_integer(x)),
									None => Err(report.error_span("modulo by zero", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::Shl => match lhs.checked_shl(rhs)
								{
									Some(x) => Ok(ExpressionValue::make_integer(x)),
									None => Err(report.error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::Shr => match lhs.checked_shr(rhs)
								{
									Some(x) => Ok(ExpressionValue::make_integer(x)),
									None => Err(report.error_span("invalid shift value", &op_span.join(&rhs_expr.span())))
								},
								
								BinaryOp::And  => Ok(ExpressionValue::make_integer(lhs & rhs)),
								BinaryOp::Or   => Ok(ExpressionValue::make_integer(lhs | rhs)),
								BinaryOp::Xor  => Ok(ExpressionValue::make_integer(lhs ^ rhs)),
								BinaryOp::Eq   => Ok(ExpressionValue::Bool(lhs == rhs)),
								BinaryOp::Ne   => Ok(ExpressionValue::Bool(lhs != rhs)),
								BinaryOp::Lt   => Ok(ExpressionValue::Bool(lhs <  rhs)),
								BinaryOp::Le   => Ok(ExpressionValue::Bool(lhs <= rhs)),
								BinaryOp::Gt   => Ok(ExpressionValue::Bool(lhs >  rhs)),
								BinaryOp::Ge   => Ok(ExpressionValue::Bool(lhs >= rhs)),
								
								BinaryOp::Concat =>
								{
									// FIXME: wrongly evaluates lhs and rhs again, with possible duplicated side-effects
									let lhs_sliced = lhs_expr.eval_slice(report.clone(), ctx, eval_var, eval_fn)?;
									let rhs_sliced = rhs_expr.eval_slice(report.clone(), ctx, eval_var, eval_fn)?;

									let (lhs_sliced, rhs_sliced) = match (lhs_sliced, rhs_sliced)
									{
										(ExpressionValue::Integer(lhs), ExpressionValue::Integer(rhs)) => (lhs, rhs),
										_ => unreachable!()
									};

									match (lhs_expr.width(), rhs_expr.width())
									{
										(Some(lhs_width), Some(rhs_width)) => Ok(ExpressionValue::make_integer(lhs_sliced.concat((lhs_width - 1, 0), &rhs_sliced, (rhs_width - 1, 0)))),
										(None, _) => Err(report.error_span("argument to concatenation with no known width", &lhs_expr.span())),
										(_, None) => Err(report.error_span("argument to concatenation with no known width", &rhs_expr.span()))
									}							
								}

								_ => Err(report.error_span("invalid argument types to operator", &span))
							}
						}
						
						(ExpressionValue::Bool(lhs), ExpressionValue::Bool(rhs)) =>
						{
							match op
							{
								BinaryOp::And => Ok(ExpressionValue::Bool(lhs & rhs)),
								BinaryOp::Or  => Ok(ExpressionValue::Bool(lhs | rhs)),
								BinaryOp::Xor => Ok(ExpressionValue::Bool(lhs ^ rhs)),
								BinaryOp::Eq  => Ok(ExpressionValue::Bool(lhs == rhs)),
								BinaryOp::Ne  => Ok(ExpressionValue::Bool(lhs != rhs)),
								_ => Err(report.error_span("invalid argument types to operator", &span))
							}
						}
						
						_ => Err(report.error_span("invalid argument types to operator", &span))
					}
				}
			}
			
			&Expression::TernaryOp(_, ref cond, ref true_branch, ref false_branch) =>
			{
				match cond.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Bool(true)  => true_branch.eval(report.clone(), ctx, eval_var, eval_fn),
					ExpressionValue::Bool(false) => false_branch.eval(report.clone(), ctx, eval_var, eval_fn),
					_ => Err(report.error_span("invalid condition type", &cond.span()))
				}
			}
			
			&Expression::BitSlice(ref span, _, left, right, ref inner) =>
			{
				match inner.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Integer(ref x) => Ok(ExpressionValue::make_integer(x.slice(left, right))),
					_ => Err(report.error_span("invalid argument type to slice", &span))
				}
			}
			
			&Expression::SoftSlice(_, _, _, _, ref inner) =>
			{
				inner.eval(report, ctx, eval_var, eval_fn)
			}
			
			&Expression::Block(_, ref exprs) =>
			{
				let mut result = ExpressionValue::Void;
				
				for expr in exprs
					{ result = expr.eval(report.clone(), ctx, eval_var, eval_fn)?; }
					
				Ok(result)
			}
			
			&Expression::Call(ref span, ref target, ref arg_exprs) =>
			{
				match target.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Function(id) =>
					{
						let mut args = Vec::new();
						for expr in arg_exprs
							{ args.push(expr.eval(report.clone(), ctx, eval_var, eval_fn)?); }
							
						match eval_fn(report, id, args, &span)
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


	pub fn eval_slice<FVar, FFn>(&self, report: RcReport, ctx: &mut ExpressionEvalContext, eval_var: &FVar, eval_fn: &FFn) -> Result<ExpressionValue, ()>
		where
		FVar: Fn(RcReport, &str, &Span) -> Result<ExpressionValue, bool>,
		FFn: Fn(RcReport, usize, Vec<ExpressionValue>, &Span) -> Result<ExpressionValue, bool>
	{
		match self
		{
			&Expression::SoftSlice(ref span, _, left, right, ref inner) =>
			{
				match inner.eval(report.clone(), ctx, eval_var, eval_fn)?
				{
					ExpressionValue::Integer(ref x) => Ok(ExpressionValue::make_integer(x.slice(left, right))),
					_ => Err(report.error_span("invalid argument type to slice", &span))
				}
			}

			_ => self.eval(report, ctx, eval_var, eval_fn)
		}
	}
}


impl ExpressionValue
{
	pub fn bits(&self) -> usize
	{
		match &self
		{
			&ExpressionValue::Integer(bigint) => bigint.min_size(),
			_ => panic!("not an integer")
		}
	}
	

	pub fn get_bit(&self, index: usize) -> bool
	{
		match self
		{
			&ExpressionValue::Integer(ref bigint) => bigint.get_bit(index),
			_ => panic!("not an integer")
		}
	}
}