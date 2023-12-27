use crate::*;


pub struct EvalContext
{
	locals: std::collections::HashMap<String, expr::Value>,
	token_substs: std::collections::HashMap<String, Vec<syntax::Token>>,
	recursion_depth: usize,
}


static ASM_HYGIENIZE_PREFIX: &'static str = ":";


impl EvalContext
{
	pub fn new() -> EvalContext
	{
		EvalContext
		{
			locals: std::collections::HashMap::new(),
			token_substs: std::collections::HashMap::new(),
			recursion_depth: 0,
		}
	}


	pub fn new_deepened(from: &EvalContext) -> EvalContext
	{
		let mut new_ctx = EvalContext::new();
		new_ctx.recursion_depth = from.recursion_depth + 1;
		new_ctx
	}


	pub fn check_recursion_depth_limit(
		&self,
		report: &mut diagn::Report,
		span: diagn::Span)
		-> Result<(), ()>
	{
		if self.recursion_depth >= expr::EVAL_RECURSION_DEPTH_MAX
		{
			report.message_with_parents_dedup(
				diagn::Message::error_span(
					"recursion depth limit reached",
					span));
	
			return Err(());
		}

		Ok(())
	}
	
	
	pub fn set_local<S>(
		&mut self,
		name: S,
		value: expr::Value)
		where S: Into<String>
	{
		self.locals.insert(name.into(), value);
	}
	
	
	pub fn get_local(
		&self,
		name: &str)
		-> Result<expr::Value, ()>
	{
		match self.locals.get(name)
		{
			Some(value) => Ok(value.clone()),
			None => Err(()),
		}
	}
	
	
	pub fn set_token_subst<S>(
		&mut self,
		name: S,
		tokens: Vec<syntax::Token>)
		where S: Into<String>
	{
		self.token_substs.insert(name.into(), tokens);
	}
	
	
	pub fn get_token_subst<'a>(
		&'a self,
		name: &str)
		-> Option<std::borrow::Cow<'a, Vec<syntax::Token>>>
	{
		if let Some(t) = self.token_substs.get(name)
		{
			return Some(std::borrow::Cow::Borrowed(t));
		}

		if let Some(_) = self.locals.get(name)
		{
			return Some(std::borrow::Cow::Owned(
				vec![syntax::Token {
					span: diagn::Span::new_dummy(),
					kind: syntax::TokenKind::Identifier,
					excerpt: Some(
						EvalContext::hygienize_name_for_asm_subst(name)),
				}]
			));
		}

		None
	}


	pub fn hygienize_locals_for_asm_subst(
		&self)
		-> EvalContext
	{
		let mut new_ctx = EvalContext::new_deepened(self);
		
		for entry in &self.locals
		{
			if entry.0.starts_with(ASM_HYGIENIZE_PREFIX)
			{
				continue;
			}

			new_ctx.locals.insert(
				EvalContext::hygienize_name_for_asm_subst(&entry.0),
				entry.1.clone());
		}
		
		for entry in &self.token_substs
		{
			if entry.0.starts_with(ASM_HYGIENIZE_PREFIX)
			{
				continue;
			}
			
			new_ctx.token_substs.insert(
				EvalContext::hygienize_name_for_asm_subst(&entry.0),
				entry.1.clone());
		}

		new_ctx
	}


	pub fn hygienize_name_for_asm_subst(
		name: &str)
		-> String
	{
		format!("{}{}",
			ASM_HYGIENIZE_PREFIX,
			name)
	}
}


pub type EvalProvider<'provider> =
	&'provider mut dyn for<'query> FnMut(EvalQuery<'query>) -> Result<expr::Value, ()>;


pub enum EvalQuery<'a>
{
	Variable(&'a mut EvalVariableQuery<'a>),
	Function(&'a mut EvalFunctionQuery<'a>),
	AsmBlock(&'a mut EvalAsmBlockQuery<'a>),
}


pub struct EvalVariableQuery<'a>
{
	pub report: &'a mut diagn::Report,
	pub hierarchy_level: usize,
	pub hierarchy: &'a Vec<String>,
	pub span: diagn::Span,
}


pub struct EvalFunctionQuery<'a>
{
	pub report: &'a mut diagn::Report,
	pub func: expr::Value,
	pub args: Vec<EvalFunctionQueryArgument>,
	pub span: diagn::Span,
	pub eval_ctx: &'a mut EvalContext,
}


impl<'a> EvalFunctionQuery<'a>
{
	pub fn ensure_arg_number(
		&mut self,
		expected_arg_number: usize)
		-> Result<(), ()>
	{
		if self.args.len() != expected_arg_number
		{
			let plural = {
				if expected_arg_number != 1
					{ "s" }
				else
					{ "" }
			};

			self.report.error_span(
				format!(
					"function expected {} argument{} (but got {})",
					expected_arg_number,
					plural,
					self.args.len()),
				self.span);
			
			Err(())
		}
		else
		{
			Ok(())
		}
	}

	pub fn ensure_min_max_arg_number(
		&mut self,
		minimum_expected_arg_number: usize,
		maximum_expected_arg_number: usize,
	) -> Result<(), ()> {
		if !((self.args.len() >= minimum_expected_arg_number)
			&& (self.args.len() <= maximum_expected_arg_number))
		{
			self.report.error_span(
				format!(
					"function expected {} to {} arguments (but got {})",
					minimum_expected_arg_number,
					maximum_expected_arg_number,
					self.args.len()
				),
				self.span,
			);
			Err(())
		} else {
			Ok(())
		}
	}
}


pub struct EvalFunctionQueryArgument
{
	pub value: expr::Value,
	pub span: diagn::Span,
}


pub struct EvalAsmBlockQuery<'a>
{
	pub report: &'a mut diagn::Report,
	pub tokens: &'a [syntax::Token],
	pub span: diagn::Span,
	pub eval_ctx: &'a mut EvalContext,
}


pub fn dummy_eval_query(
	query: expr::EvalQuery)
	-> Result<expr::Value, ()>
{
	match query
	{
		expr::EvalQuery::Variable(query_var) =>
			expr::dummy_eval_var(query_var),
		
		expr::EvalQuery::Function(query_fn) =>
			expr::dummy_eval_fn(query_fn),
			
		expr::EvalQuery::AsmBlock(query_asm) =>
			expr::dummy_eval_asm(query_asm),
	}
}


pub fn dummy_eval_var(
	query: &mut EvalVariableQuery)
	-> Result<expr::Value, ()>
{
	query.report.error_span(
		"cannot reference variables in this context",
		query.span);
		
	Err(())
}


pub fn dummy_eval_fn(
	query: &mut EvalFunctionQuery)
	-> Result<expr::Value, ()>
{
	query.report.error_span(
		"cannot reference functions in this context",
		query.span);
		
	Err(())
}


pub fn dummy_eval_asm(
	query: &mut EvalAsmBlockQuery)
	-> Result<expr::Value, ()>
{
	query.report.error_span(
		"cannot use `asm` blocks in this context",
		query.span);
		
	Err(())
}


macro_rules! propagate {
	($expr: expr) => {
		{
			let value: expr::Value = $expr;

			if value.should_propagate()
			{
				return Ok(value)
			}
			else
			{
				value
			}
		}
	};
}


impl expr::Expr
{
	pub fn eval_usize<'provider>(
		&self,
		report: &mut diagn::Report,
		provider: EvalProvider)
		-> Result<usize, ()>
	{
        self
			.eval_with_ctx(
				report,
				&mut EvalContext::new(),
				provider)?
			.expect_usize(
				report,
				self.span())
	}


	pub fn eval_nonzero_usize<'provider>(
		&self,
		report: &mut diagn::Report,
		provider: EvalProvider<'provider>)
		-> Result<usize, ()>
	{
        self
			.eval_with_ctx(
				report,
				&mut EvalContext::new(),
				provider)?
			.expect_nonzero_usize(
				report,
				self.span())
	}


	pub fn eval_bigint<'provider>(
		&self,
		report: &mut diagn::Report,
		provider: EvalProvider<'provider>)
		-> Result<util::BigInt, ()>
	{
        let result = self.eval_with_ctx(
			report,
			&mut EvalContext::new(),
			provider)?;

		let bigint = result.expect_bigint(
			report,
			self.span())?;

		Ok(bigint.clone())
	}


	pub fn eval<'provider>(
		&self,
		report: &mut diagn::Report,
		provider: EvalProvider<'provider>)
		-> Result<expr::Value, ()>
	{
        self.eval_with_ctx(
            report,
            &mut EvalContext::new(),
            provider)
    }


	pub fn eval_with_ctx<'provider>(
		&self,
		report: &mut diagn::Report,
		ctx: &mut EvalContext,
		provider: EvalProvider<'provider>)
		-> Result<expr::Value, ()>
	{
		match self
		{
			&expr::Expr::Literal(_, ref value) => Ok(value.clone()),
			
			&expr::Expr::Variable(span, hierarchy_level, ref hierarchy) =>
			{
				let mut query = EvalVariableQuery {
					report,
					hierarchy_level,
					hierarchy,
					span,
				};

				if hierarchy_level == 0 && hierarchy.len() == 1
				{
					if let Some(_) = expr::resolve_builtin_fn(&hierarchy[0])
					{
						return Ok(expr::Value::ExprBuiltInFunction(
							hierarchy[0].clone()));
					}

					if let Ok(local_value) = ctx.get_local(&hierarchy[0])
					{
						return Ok(local_value);
					}
				}

				provider(EvalQuery::Variable(&mut query))
			}
			
			&expr::Expr::UnaryOp(span, _, op, ref inner_expr) =>
			{
				match propagate!(
					inner_expr.eval_with_ctx(report, ctx, provider)?)
				{
					expr::Value::Integer(ref x) => match op
					{
						expr::UnaryOp::Neg => Ok(expr::Value::make_integer(-x)),
						expr::UnaryOp::Not => Ok(expr::Value::make_integer(!x))
					},
					
					expr::Value::Bool(b) => match op
					{
						expr::UnaryOp::Not => Ok(expr::Value::Bool(!b)),
						_ => Err(report.error_span("invalid argument type to operator", span))
					},
					
					_ => Err(report.error_span("invalid argument type to operator", span))
				}
			}
			
			&expr::Expr::BinaryOp(span, _, op, ref lhs_expr, ref rhs_expr) =>
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
								let value = propagate!(
									rhs_expr.eval_with_ctx(report, ctx, provider)?);
								ctx.set_local(hierarchy[0].clone(), value);
								return Ok(expr::Value::Void);
							}
							
							Err(report.error_span("symbol cannot be assigned to", lhs_expr.span()))
						}
						
						_ => Err(report.error_span("invalid assignment destination", lhs_expr.span()))
					}
				}
				
				else if op == expr::BinaryOp::LazyOr || op == expr::BinaryOp::LazyAnd
				{
					let lhs = propagate!(
						lhs_expr.eval_with_ctx(report, ctx, provider)?);
					
					match (op, &lhs)
					{
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(true))  => return Ok(lhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(false)) => return Ok(lhs),
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(false)) => { }
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(true))  => { }
						_ => return Err(report.error_span("invalid argument type to operator", lhs_expr.span()))
					}
					
					let rhs = propagate!(
						rhs_expr.eval_with_ctx(report, ctx, provider)?);
					
					match (op, &rhs)
					{
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(true))  => Ok(rhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(false)) => Ok(rhs),
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(false)) => Ok(rhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(true))  => Ok(rhs),
						_ => Err(report.error_span("invalid argument type to operator", rhs_expr.span()))
					}
				}
				
				else
				{
					let lhs = propagate!(
						lhs_expr.eval_with_ctx(report, ctx, provider)?);

					let rhs = propagate!(
						rhs_expr.eval_with_ctx(report, ctx, provider)?);

					match (&lhs, &rhs)
					{
						(expr::Value::Bool(lhs), expr::Value::Bool(rhs)) =>
						{
							return match op
							{
								expr::BinaryOp::And => Ok(expr::Value::Bool(lhs & rhs)),
								expr::BinaryOp::Or  => Ok(expr::Value::Bool(lhs | rhs)),
								expr::BinaryOp::Xor => Ok(expr::Value::Bool(lhs ^ rhs)),
								expr::BinaryOp::Eq  => Ok(expr::Value::Bool(lhs == rhs)),
								expr::BinaryOp::Ne  => Ok(expr::Value::Bool(lhs != rhs)),
								_ => Err(report.error_span("invalid argument types to operator", span))
							}
						}

						_ => {}
					}
					
					let lhs_bigint = lhs.get_bigint();
					let rhs_bigint = rhs.get_bigint();

					match (lhs_bigint, rhs_bigint)
					{
						(Some(ref lhs), Some(ref rhs)) =>
						{
							match op
							{
								expr::BinaryOp::Add =>
									Ok(expr::Value::make_integer(
										lhs.checked_add(
											report,
											span,
											rhs)?)),

								expr::BinaryOp::Sub =>
									Ok(expr::Value::make_integer(
										lhs.checked_sub(
											report,
											span,
											rhs)?)),

								expr::BinaryOp::Mul =>
									Ok(expr::Value::make_integer(
										lhs.checked_mul(
											report,
											span,
											rhs)?)),
								
								expr::BinaryOp::Div =>
									Ok(expr::Value::make_integer(
										lhs.checked_div(
											report,
											span,
											rhs)?)),
								
								expr::BinaryOp::Mod =>
									Ok(expr::Value::make_integer(
										lhs.checked_mod(
											report,
											span,
											rhs)?)),
								
								expr::BinaryOp::Shl =>
									Ok(expr::Value::make_integer(
										lhs.checked_shl(
											report,
											span,
											rhs)?)),
								
								expr::BinaryOp::Shr =>
									Ok(expr::Value::make_integer(
										lhs.checked_shr(
											report,
											span,
											rhs)?)),
								
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
									match (lhs.size, rhs.size)
									{
										(Some(lhs_width), Some(rhs_width)) => Ok(expr::Value::make_integer(lhs.concat((lhs_width, 0), &rhs, (rhs_width, 0)))),
										(None, _) => Err(report.error_span("argument to concatenation with indefinite size", lhs_expr.span())),
										(_, None) => Err(report.error_span("argument to concatenation with indefinite size", rhs_expr.span()))
									}
								}

								_ => Err(report.error_span("invalid argument types to operator", span))
							}
						}
						
						_ => Err(report.error_span("invalid argument types to operator", span))
					}
				}
			}
			
			&expr::Expr::TernaryOp(_, ref cond, ref true_branch, ref false_branch) =>
			{
				match cond.eval_with_ctx(report, ctx, provider)?
				{
					expr::Value::Bool(true)  => Ok(propagate!(
						true_branch.eval_with_ctx(report, ctx, provider)?)),
					expr::Value::Bool(false) => Ok(propagate!(
						false_branch.eval_with_ctx(report, ctx, provider)?)),
					_ => Err(report.error_span("invalid condition type", cond.span()))
				}
			}
			
			&expr::Expr::BitSlice(span, _, left, right, ref inner) =>
			{
				match propagate!(
					inner.eval_with_ctx(report, ctx, provider)?).get_bigint()
				{
					Some(ref x) => Ok(expr::Value::make_integer(x.slice(left, right))),
					None => Err(report.error_span("invalid argument type to slice", span))
				}
			}
			
			&expr::Expr::Block(_, ref exprs) =>
			{
				let mut result = expr::Value::Void;
				
				for expr in exprs
				{
					result = propagate!(
						expr.eval_with_ctx(report, ctx, provider)?);
				}
					
				Ok(result)
			}
			
			&expr::Expr::Call(span, ref target, ref arg_exprs) =>
			{
				let func = propagate!(
					target.eval_with_ctx(report, ctx, provider)?);

				let mut args = Vec::with_capacity(arg_exprs.len());
				for expr in arg_exprs
				{
					let value = propagate!(
						expr.eval_with_ctx(report, ctx, provider)?);
					
					args.push(EvalFunctionQueryArgument {
						value,
						span: expr.span()
					});
				}

				let mut query = EvalFunctionQuery {
					report,
					func,
					args,
					span,
					eval_ctx: ctx,
				};

				match query.func
				{
					expr::Value::ExprBuiltInFunction(_) =>
						expr::eval_builtin_fn(&mut query),

					expr::Value::AsmBuiltInFunction(_) =>
						provider(EvalQuery::Function(&mut query)),

					expr::Value::Function(_) =>
						provider(EvalQuery::Function(&mut query)),

					expr::Value::Unknown =>
						Err(report.error_span("unknown function", target.span())),
					
					_ =>
						Err(report.error_span("expression is not callable", target.span()))
				}
			}
			
			&expr::Expr::Asm(span, ref tokens) =>
			{
				let mut query = EvalAsmBlockQuery {
					report,
					tokens,
					span,
					eval_ctx: ctx,
				};

                provider(EvalQuery::AsmBlock(&mut query))
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