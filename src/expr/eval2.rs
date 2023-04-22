use crate::*;


pub struct EvalContext2
{
	locals: std::collections::HashMap<String, expr::Value>,
	token_substs: std::collections::HashMap<String, Vec<syntax::Token>>,
	pub eval_asm_depth: usize,
}


static ASM_HYGIENIZE_PREFIX: &'static str = ":";


impl EvalContext2
{
	pub fn new() -> EvalContext2
	{
		EvalContext2
		{
			locals: std::collections::HashMap::new(),
			token_substs: std::collections::HashMap::new(),
			eval_asm_depth: 0,
		}
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
						EvalContext2::hygienize_name_for_asm_subst(name)),
				}]
			));
		}

		None
	}


	pub fn hygienize_locals_for_asm_subst(
		&self)
		-> EvalContext2
	{
		let mut new_ctx = EvalContext2::new();
		new_ctx.eval_asm_depth = self.eval_asm_depth;
		
		for entry in &self.locals
		{
			if entry.0.starts_with(ASM_HYGIENIZE_PREFIX)
			{
				continue;
			}

			new_ctx.locals.insert(
				EvalContext2::hygienize_name_for_asm_subst(&entry.0),
				entry.1.clone());
		}
		
		for entry in &self.token_substs
		{
			if entry.0.starts_with(ASM_HYGIENIZE_PREFIX)
			{
				continue;
			}
			
			new_ctx.token_substs.insert(
				EvalContext2::hygienize_name_for_asm_subst(&entry.0),
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


pub struct EvalVariableInfo2<'a>
{
	pub report: &'a mut diagn::Report,
	pub hierarchy_level: usize,
	pub hierarchy: &'a Vec<String>,
	pub span: &'a diagn::Span,
}


pub struct EvalFunctionInfo2<'a>
{
	pub report: &'a mut diagn::Report,
	pub func: expr::Value,
	pub args: Vec<EvalFunctionArgument<'a>>,
	pub span: &'a diagn::Span,
}


impl<'a> EvalFunctionInfo2<'a>
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
}


pub struct EvalFunctionArgument<'a>
{
	pub value: expr::Value,
	pub span: &'a diagn::Span,
}


pub struct EvalAsmInfo2<'a>
{
	pub report: &'a mut diagn::Report,
	pub tokens: &'a [syntax::Token],
	pub span: &'a diagn::Span,
	pub eval_ctx: &'a mut EvalContext2,
}


pub struct EvalProvider<'f, FVar, FFn, FAsm>
where
    FVar: FnMut(&mut EvalVariableInfo2) -> Result<expr::Value, ()>,
    FFn: FnMut(&mut EvalFunctionInfo2) -> Result<expr::Value, ()>,
    FAsm: FnMut(&mut EvalAsmInfo2) -> Result<expr::Value, ()>
{
    pub eval_var: &'f mut FVar,
    pub eval_fn: &'f mut FFn,
    pub eval_asm: &'f mut FAsm,
}


pub fn dummy_eval_var() -> impl Fn(&mut EvalVariableInfo2) -> Result<expr::Value, ()>
{
    |info| {
        info.report.error_span(
            "cannot reference variables in this context",
            &info.span);
            
        Err(())
    }
}


pub fn dummy_eval_fn() -> impl Fn(&mut EvalFunctionInfo2) -> Result<expr::Value, ()>
{
    |info| {
        info.report.error_span(
            "cannot reference functions in this context",
            &info.span);
            
        Err(())
    }
}


pub fn dummy_eval_asm() -> impl Fn(&mut EvalAsmInfo2) -> Result<expr::Value, ()>
{
    |info| {
        info.report.error_span(
            "cannot use `asm` blocks in this context",
            &info.span);
            
        Err(())
    }
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
	pub fn eval_usize<'f, FVar, FFn, FAsm>(
		&self,
		report: &mut diagn::Report,
		provider: &mut EvalProvider<'f, FVar, FFn, FAsm>)
		-> Result<usize, ()>
        where
            FVar: FnMut(&mut EvalVariableInfo2) -> Result<expr::Value, ()>,
            FFn: FnMut(&mut EvalFunctionInfo2) -> Result<expr::Value, ()>,
            FAsm: FnMut(&mut EvalAsmInfo2) -> Result<expr::Value, ()>
	{
        self
			.eval2_with_ctx(
				report,
				&mut EvalContext2::new(),
				provider)?
			.expect_usize(
				report,
				&self.span())
	}


	pub fn eval_nonzero_usize<'f, FVar, FFn, FAsm>(
		&self,
		report: &mut diagn::Report,
		provider: &mut EvalProvider<'f, FVar, FFn, FAsm>)
		-> Result<usize, ()>
        where
            FVar: FnMut(&mut EvalVariableInfo2) -> Result<expr::Value, ()>,
            FFn: FnMut(&mut EvalFunctionInfo2) -> Result<expr::Value, ()>,
            FAsm: FnMut(&mut EvalAsmInfo2) -> Result<expr::Value, ()>
	{
        self
			.eval2_with_ctx(
				report,
				&mut EvalContext2::new(),
				provider)?
			.expect_nonzero_usize(
				report,
				&self.span())
	}


	pub fn eval_bigint<'f, FVar, FFn, FAsm>(
		&self,
		report: &mut diagn::Report,
		provider: &mut EvalProvider<'f, FVar, FFn, FAsm>)
		-> Result<util::BigInt, ()>
        where
            FVar: FnMut(&mut EvalVariableInfo2) -> Result<expr::Value, ()>,
            FFn: FnMut(&mut EvalFunctionInfo2) -> Result<expr::Value, ()>,
            FAsm: FnMut(&mut EvalAsmInfo2) -> Result<expr::Value, ()>
	{
        let result = self.eval2_with_ctx(
			report,
			&mut EvalContext2::new(),
			provider)?;

		let bigint = result.expect_bigint(
			report,
			&self.span())?;

		Ok(bigint.clone())
	}


	pub fn eval2<'f, FVar, FFn, FAsm>(
		&self,
		report: &mut diagn::Report,
		provider: &mut EvalProvider<'f, FVar, FFn, FAsm>)
		-> Result<expr::Value, ()>
        where
            FVar: FnMut(&mut EvalVariableInfo2) -> Result<expr::Value, ()>,
            FFn: FnMut(&mut EvalFunctionInfo2) -> Result<expr::Value, ()>,
            FAsm: FnMut(&mut EvalAsmInfo2) -> Result<expr::Value, ()>
	{
        self.eval2_with_ctx(
            report,
            &mut EvalContext2::new(),
            provider)
    }


	pub fn eval2_with_ctx<'f, FVar, FFn, FAsm>(
		&self,
		report: &mut diagn::Report,
		ctx: &mut EvalContext2,
		provider: &mut EvalProvider<'f, FVar, FFn, FAsm>)
		-> Result<expr::Value, ()>
        where
            FVar: FnMut(&mut EvalVariableInfo2) -> Result<expr::Value, ()>,
            FFn: FnMut(&mut EvalFunctionInfo2) -> Result<expr::Value, ()>,
            FAsm: FnMut(&mut EvalAsmInfo2) -> Result<expr::Value, ()>
	{
		match self
		{
			&expr::Expr::Literal(_, ref value) => Ok(value.clone()),
			
			&expr::Expr::Variable(ref span, hierarchy_level, ref hierarchy) =>
			{
				let mut info = EvalVariableInfo2
				{
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

				(provider.eval_var)(&mut info)
			}
			
			&expr::Expr::UnaryOp(ref span, _, op, ref inner_expr) =>
			{
				match propagate!(
					inner_expr.eval2_with_ctx(report, ctx, provider)?)
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
								let value = propagate!(
									rhs_expr.eval2_with_ctx(report, ctx, provider)?);
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
					let lhs = propagate!(
						lhs_expr.eval2_with_ctx(report, ctx, provider)?);
					
					match (op, &lhs)
					{
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(true))  => return Ok(lhs),
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(false)) => return Ok(lhs),
						(expr::BinaryOp::LazyOr,  &expr::Value::Bool(false)) => { }
						(expr::BinaryOp::LazyAnd, &expr::Value::Bool(true))  => { }
						_ => return Err(report.error_span("invalid argument type to operator", &lhs_expr.span()))
					}
					
					let rhs = propagate!(
						rhs_expr.eval2_with_ctx(report, ctx, provider)?);
					
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
					let lhs = propagate!(
						lhs_expr.eval2_with_ctx(report, ctx, provider)?);

					let rhs = propagate!(
						rhs_expr.eval2_with_ctx(report, ctx, provider)?);

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
								_ => Err(report.error_span("invalid argument types to operator", &span))
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
									match (lhs.size, rhs.size)
									{
										(Some(lhs_width), Some(rhs_width)) => Ok(expr::Value::make_integer(lhs.concat((lhs_width, 0), &rhs, (rhs_width, 0)))),
										(None, _) => Err(report.error_span("argument to concatenation with indefinite size", &lhs_expr.span())),
										(_, None) => Err(report.error_span("argument to concatenation with indefinite size", &rhs_expr.span()))
									}
								}

								_ => Err(report.error_span("invalid argument types to operator", &span))
							}
						}
						
						_ => Err(report.error_span("invalid argument types to operator", &span))
					}
				}
			}
			
			&expr::Expr::TernaryOp(_, ref cond, ref true_branch, ref false_branch) =>
			{
				match cond.eval2_with_ctx(report, ctx, provider)?
				{
					expr::Value::Bool(true)  => Ok(propagate!(
						true_branch.eval2_with_ctx(report, ctx, provider)?)),
					expr::Value::Bool(false) => Ok(propagate!(
						false_branch.eval2_with_ctx(report, ctx, provider)?)),
					_ => Err(report.error_span("invalid condition type", &cond.span()))
				}
			}
			
			&expr::Expr::BitSlice(ref span, _, left, right, ref inner) =>
			{
				match propagate!(
					inner.eval2_with_ctx(report, ctx, provider)?).get_bigint()
				{
					Some(ref x) => Ok(expr::Value::make_integer(x.slice(left, right))),
					None => Err(report.error_span("invalid argument type to slice", &span))
				}
			}
			
			&expr::Expr::SoftSlice(_, _, _, _, ref inner) =>
			{
				inner.eval2_with_ctx(report, ctx, provider)
			}
			
			&expr::Expr::Block(_, ref exprs) =>
			{
				let mut result = expr::Value::Void;
				
				for expr in exprs
				{
					result = propagate!(
						expr.eval2_with_ctx(report, ctx, provider)?);
				}
					
				Ok(result)
			}
			
			&expr::Expr::Call(ref span, ref target, ref arg_exprs) =>
			{
				let func = propagate!(
					target.eval2_with_ctx(report, ctx, provider)?);

				let mut args = Vec::with_capacity(arg_exprs.len());
				for expr in arg_exprs
				{
					let value = propagate!(
						expr.eval2_with_ctx(report, ctx, provider)?);
					
					args.push(EvalFunctionArgument {
						value,
						span: expr.span()
					});
				}

				let mut info = EvalFunctionInfo2 {
					report,
					func,
					args,
					span,
				};

				match info.func
				{
					expr::Value::ExprBuiltInFunction(_) =>
						expr::eval_builtin_fn(&mut info),

					expr::Value::AsmBuiltInFunction(_) =>
						(provider.eval_fn)(&mut info),

					expr::Value::Function(_) =>
						(provider.eval_fn)(&mut info),

					expr::Value::Unknown =>
						Err(report.error_span("unknown function", &target.span())),
					
					_ =>
						Err(report.error_span("expression is not callable", &target.span()))
				}
			}
			
			&expr::Expr::Asm(ref span, ref tokens) =>
			{
				let mut info = EvalAsmInfo2 {
					report,
					tokens,
					span,
					eval_ctx: ctx,
				};

                (provider.eval_asm)(&mut info)
			}
		}
	}
}