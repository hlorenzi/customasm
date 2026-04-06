use crate::*;


pub struct EvalContext<'opts>
{
	pub opts: &'opts asm::AssemblyOptions,
	locals: std::collections::HashMap<String, expr::Value>,
	token_substs: std::collections::HashMap<String, String>,
	recursion_depth: usize,
}


pub static ASM_SUBSTITUTION_VARIABLE: &'static str = "$subst";


impl<'opts> EvalContext<'opts>
{
	pub fn new(opts: &'opts asm::AssemblyOptions) -> EvalContext<'opts>
	{
		EvalContext
		{
			opts,
			locals: std::collections::HashMap::new(),
			token_substs: std::collections::HashMap::new(),
			recursion_depth: 0,
		}
	}


	pub fn new_deepened<'a>(from: &'a EvalContext<'opts>) -> EvalContext<'opts>
	{
		let mut new_ctx = EvalContext::new(from.opts);
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
		-> Option<expr::Value>
	{
		match self.locals.get(name)
		{
			Some(value) => Some(value.clone()),
			None => None,
		}
	}
	
	
	pub fn set_token_subst<S>(
		&mut self,
		name: S,
		excerpt: String)
		where S: Into<String>
	{
		self.token_substs.insert(name.into(), excerpt);
	}
	
	
	pub fn get_token_subst<'a>(
		&'a self,
		name: &str)
		-> Option<std::borrow::Cow<'a, String>>
	{
		if let Some(t) = self.token_substs.get(name)
		{
			return Some(std::borrow::Cow::Borrowed(t));
		}

		None
	}


	pub fn new_asm_subst(&self) -> String
	{
		format!(
			"{}{}",
			expr::ASM_SUBSTITUTION_VARIABLE,
			self.locals.len())
	}
}


pub type EvalProvider<'provider> =
	&'provider mut dyn for<'query, 'opts> FnMut(EvalQuery<'query, 'opts>) -> Result<expr::Value, ()>;


pub enum EvalQuery<'a, 'opts>
{
	CtxLabel(&'a mut EvalCtxLabelQuery<'a>),
	Variable(&'a mut EvalVariableQuery<'a>),
	Member(&'a mut EvalMemberQuery<'a>),
	Function(&'a mut EvalFunctionQuery<'a, 'opts>),
	AsmBlock(&'a mut EvalAsmBlockQuery<'a, 'opts>),
}


pub struct EvalCtxLabelQuery<'a>
{
	pub report: &'a mut diagn::Report,
	pub nesting_level: usize,
	pub span: diagn::Span,
}


pub struct EvalVariableQuery<'a>
{
	pub report: &'a mut diagn::Report,
	pub opts: &'a asm::AssemblyOptions,
	pub hierarchy_level: usize,
	pub hierarchy: &'a Vec<String>,
	pub span: diagn::Span,
}


pub struct EvalMemberQuery<'a>
{
	pub report: &'a mut diagn::Report,
	pub value: expr::Value,
	pub member_name: &'a str,
	pub span: diagn::Span,
}


pub struct EvalFunctionQuery<'a, 'opts>
{
	pub report: &'a mut diagn::Report,
	pub func: expr::Value,
	pub args: Vec<EvalFunctionQueryArgument>,
	pub span: diagn::Span,
	pub eval_ctx: &'a mut EvalContext<'opts>,
}


impl<'a, 'opts> EvalFunctionQuery<'a, 'opts>
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


pub struct EvalAsmBlockQuery<'a, 'opts>
{
	pub report: &'a mut diagn::Report,
	pub ast: &'a asm::AstTopLevel,
	pub span: diagn::Span,
	pub eval_ctx: &'a mut EvalContext<'opts>,
}


pub fn dummy_eval_query(
	query: expr::EvalQuery)
	-> Result<expr::Value, ()>
{
	match query
	{
		expr::EvalQuery::CtxLabel(query_ctxlabel) =>
			expr::dummy_eval_ctxlabel(query_ctxlabel),
		
		expr::EvalQuery::Variable(query_var) =>
			expr::dummy_eval_var(query_var),
		
		expr::EvalQuery::Member(query_member) =>
			expr::dummy_eval_member(query_member),
		
		expr::EvalQuery::Function(query_fn) =>
			expr::dummy_eval_fn(query_fn),
			
		expr::EvalQuery::AsmBlock(query_asm) =>
			expr::dummy_eval_asm(query_asm),
	}
}


pub fn dummy_eval_ctxlabel(
	query: &mut EvalCtxLabelQuery)
	-> Result<expr::Value, ()>
{
	query.report.error_span(
		"cannot reference contextual labels in this context",
		query.span);
		
	Err(())
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


pub fn dummy_eval_member(
	query: &mut EvalMemberQuery)
	-> Result<expr::Value, ()>
{
	if let Some(value) = expr::resolve_builtin_member(query)?
	{
		return Ok(value);
	}

	query.report.error_span(
		"cannot access members in this context",
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
	pub fn try_eval_usize<'provider>(
		&self,
		opts: &asm::AssemblyOptions)
		-> Option<usize>
	{
        let value = self.eval_with_ctx(
			&mut diagn::Report::new(),
			&mut EvalContext::new(opts),
			&mut dummy_eval_query);

		if let Ok(expr::Value::Integer(_, bigint)) = value
		{
			bigint.maybe_into::<usize>()
		}
		else
		{
			None
		}
	}


	pub fn eval_nonzero_usize<'provider>(
		&self,
		report: &mut diagn::Report,
		opts: &asm::AssemblyOptions,
		provider: EvalProvider<'provider>)
		-> Result<usize, ()>
	{
        self
			.eval_with_ctx(
				report,
				&mut EvalContext::new(opts),
				provider)?
			.expect_nonzero_usize(
				report,
				self.span())
	}


	pub fn eval_bigint<'provider>(
		&self,
		report: &mut diagn::Report,
		opts: &asm::AssemblyOptions,
		provider: EvalProvider<'provider>)
		-> Result<util::BigInt, ()>
	{
        let result = self.eval_with_ctx(
			report,
			&mut EvalContext::new(opts),
			provider)?;

		let bigint = result.expect_bigint(
			report,
			self.span())?;

		Ok(bigint.clone())
	}


	pub fn eval<'provider>(
		&self,
		report: &mut diagn::Report,
		opts: &asm::AssemblyOptions,
		provider: EvalProvider<'provider>)
		-> Result<expr::Value, ()>
	{
        self.eval_with_ctx(
            report,
            &mut EvalContext::new(opts),
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
			
			&expr::Expr::Variable(span, ref name) =>
			{
				if let Some(builtin_fn) = expr::resolve_builtin_fn(name, ctx.opts)
				{
					return Ok(expr::Value::ExprBuiltinFn(expr::Value::make_metadata(), builtin_fn));
				}

				if let Some(local_value) = ctx.get_local(name)
				{
					return Ok(local_value);
				}

				let mut query = EvalVariableQuery {
					report,
					span,
					opts: ctx.opts,
					hierarchy_level: 0,
					hierarchy: &vec![name.clone()],
				};

				provider(EvalQuery::Variable(&mut query))
			}

			&expr::Expr::StructInit { ref members_init, .. } =>
			{
				let mut metadata = expr::Value::make_unknown().statically_known();

				let mut members = Vec::with_capacity(members_init.len());
				for member_init in members_init
				{
					let value = propagate!(member_init.value
						.eval_with_ctx(report, ctx, provider)?);
					
					metadata = metadata.derived_from(&value);

					members.push(expr::ValueStructMember {
						name: member_init.name.clone(),
						value,
					});
				}

				Ok(expr::Value::Struct(
					*metadata.get_metadata(),
					expr::ValueStruct {
						members,
					}))
			}

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
			}
			
			&expr::Expr::UnaryOp(span, _, op, ref inner_expr) =>
			{
				let inner = propagate!(inner_expr
					.eval_with_ctx(report, ctx, provider)?);

				match inner
				{
					expr::Value::Integer(_, ref x) => match op
					{
						expr::UnaryOp::Neg => Ok(expr::Value::make_integer(-x).statically_known().derived_from(&inner)),
						expr::UnaryOp::Not => Ok(expr::Value::make_integer(!x).statically_known().derived_from(&inner))
					},
					
					expr::Value::Bool(_, b) => match op
					{
						expr::UnaryOp::Not => Ok(expr::Value::make_bool(!b).statically_known().derived_from(&inner)),
						_ => Err(report.error_span(
								format!(
									"invalid argument type to operator (have {})",
									inner.type_name()),
								span))
					},
					
					_ => Err(report.error_span(
							format!(
								"invalid argument type to operator (have {})",
								inner.type_name()),
							span))
				}
			}
			
			&expr::Expr::BinaryOp(span, _, op, ref lhs_expr, ref rhs_expr) =>
				eval_binary_op(report, ctx, provider, lhs_expr, rhs_expr, op, span),
			
			&expr::Expr::TernaryOp(_, ref cond, ref true_branch, ref false_branch) =>
			{
				match propagate!(cond
					.eval_with_ctx(report, ctx, provider)?)
				{
					expr::Value::Bool(_, true)  => Ok(propagate!(
						true_branch.eval_with_ctx(report, ctx, provider)?)),
					expr::Value::Bool(_, false) => Ok(propagate!(
						false_branch.eval_with_ctx(report, ctx, provider)?)),
					_ => Err(report.error_span("invalid condition type", cond.span()))
				}
			}
			
			&expr::Expr::Slice(span, _, ref left_expr, ref right_expr, ref inner_expr) =>
			{
				let inner = propagate!(
					inner_expr.eval_with_ctx(report, ctx, provider)?);
				
				match inner
				{
					expr::Value::Integer(_, ref inner_int) =>
					{
						let left = propagate!(left_expr
							.eval_with_ctx(report, ctx, provider)?);

						let right = propagate!(right_expr
							.eval_with_ctx(report, ctx, provider)?);

						let left_usize = left.expect_usize(report, left_expr.span())? + 1;
						let right_usize = right.expect_usize(report, right_expr.span())?;

						Ok(expr::Value::make_integer(
								inner_int.checked_slice(report, span, left_usize, right_usize)?)
							.statically_known()
							.derived_from(&inner)
							.derived_from(&left)
							.derived_from(&right))
					}
					_ => Err(report.error_span(
						format!(
							"invalid argument type to slice (have {})",
							inner.type_name()),
						inner_expr.span()))
				}
			}
			
			&expr::Expr::SliceShort(span, size_span, ref size_expr, ref inner_expr) =>
			{
				let inner = propagate!(
					inner_expr.eval_with_ctx(report, ctx, provider)?);
				
				match inner
				{
					expr::Value::Integer(_, ref inner_int) =>
					{
						let size = propagate!(size_expr
							.eval_with_ctx(report, ctx, provider)?);

						let size_usize = size.expect_usize(report, size_span)?;
						
						Ok(expr::Value::make_integer(
								inner_int.checked_slice(report, span, size_usize, 0)?)
							.statically_known()
							.derived_from(&inner)
							.derived_from(&size))
					}
					_ => Err(report.error_span(
						format!(
							"invalid argument type to slice (have {})",
							inner.type_name()),
						inner_expr.span()))
				}
			}
			
			&expr::Expr::Block(_, ref exprs) =>
			{
				let mut result = expr::Value::make_void().statically_known();
				
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
					expr::Value::ExprBuiltinFn(_, builtin_fn) =>
						expr::get_builtin_fn_eval(builtin_fn)(&mut query),

					expr::Value::AsmBuiltinFn(_, _) =>
						provider(EvalQuery::Function(&mut query)),

					expr::Value::Function(_, _) =>
						provider(EvalQuery::Function(&mut query)),

					expr::Value::Unknown(_) =>
						Err(report.error_span("unknown function", target.span())),
					
					_ =>
						Err(report.error_span("expression is not callable", target.span()))
				}
			}
			
			&expr::Expr::Asm(span, ref ast) =>
			{
				let mut query = EvalAsmBlockQuery {
					report,
					ast,
					span,
					eval_ctx: ctx,
				};

                provider(EvalQuery::AsmBlock(&mut query))
			}
		}
	}
}


fn eval_binary_op<'provider>(
	report: &mut diagn::Report,
	ctx: &mut EvalContext,
	provider: EvalProvider<'provider>,
	lhs_expr: &expr::Expr,
	rhs_expr: &expr::Expr,
	op: expr::BinaryOp,
	span: diagn::Span)
	-> Result<expr::Value, ()>
{
	if op == expr::BinaryOp::Assign
	{
		match lhs_expr
		{
			&expr::Expr::Variable(span, ref name) =>
			{
				asm::check_reserved_name(
					report,
					span,
					ctx.opts,
					name)?;

				let value = propagate!(rhs_expr
					.eval_with_ctx(report, ctx, provider)?);
				
				ctx.set_local(name.clone(), value);
				return Ok(expr::Value::make_void().statically_known());
			}
			
			_ => return Err(report.error_span("invalid assignment destination", lhs_expr.span()))
		}
	}
	
	else if op == expr::BinaryOp::LazyOr || op == expr::BinaryOp::LazyAnd
	{
		let lhs = propagate!(lhs_expr
			.eval_with_ctx(report, ctx, provider)?);
		
		match (op, &lhs)
		{
			(expr::BinaryOp::LazyOr,  &expr::Value::Bool(_, true))  => return Ok(lhs),
			(expr::BinaryOp::LazyAnd, &expr::Value::Bool(_, false)) => return Ok(lhs),
			(expr::BinaryOp::LazyOr,  &expr::Value::Bool(_, false)) => { }
			(expr::BinaryOp::LazyAnd, &expr::Value::Bool(_, true))  => { }
			_ => return Err(report.error_span("invalid argument type to operator", lhs_expr.span()))
		}
		
		let rhs = propagate!(rhs_expr
			.eval_with_ctx(report, ctx, provider)?);
		
		match (op, &rhs)
		{
			(expr::BinaryOp::LazyOr,  &expr::Value::Bool(_, true))  => return Ok(rhs.statically_known().derived_from(&lhs)),
			(expr::BinaryOp::LazyAnd, &expr::Value::Bool(_, false)) => return Ok(rhs.statically_known().derived_from(&lhs)),
			(expr::BinaryOp::LazyOr,  &expr::Value::Bool(_, false)) => return Ok(rhs.statically_known().derived_from(&lhs)),
			(expr::BinaryOp::LazyAnd, &expr::Value::Bool(_, true))  => return Ok(rhs.statically_known().derived_from(&lhs)),
			_ => return Err(report.error_span("invalid argument type to operator", rhs_expr.span()))
		}
	}

	let lhs = propagate!(lhs_expr.eval_with_ctx(report, ctx, provider)?);
	let rhs = propagate!(rhs_expr.eval_with_ctx(report, ctx, provider)?);

	match (op, &lhs, &rhs)
	{
		(expr::BinaryOp::Eq, lhs, rhs)
		if std::mem::discriminant(lhs) == std::mem::discriminant(rhs) => {
			return Ok(expr::Value::make_bool(lhs == rhs).statically_known().derived_from(lhs).derived_from(rhs))
		}
		(expr::BinaryOp::Ne, lhs, rhs) =>
		if std::mem::discriminant(lhs) == std::mem::discriminant(rhs) {
			return Ok(expr::Value::make_bool(lhs != rhs).statically_known().derived_from(lhs).derived_from(rhs))
		}
		(expr::BinaryOp::And, expr::Value::Bool(_, lhs_bool), expr::Value::Bool(_, rhs_bool)) => {
			return Ok(expr::Value::make_bool(lhs_bool & rhs_bool).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::And, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int & rhs_int).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Or, expr::Value::Bool(_, lhs_bool), expr::Value::Bool(_, rhs_bool)) => {
			return Ok(expr::Value::make_bool(lhs_bool | rhs_bool).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Or, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int | rhs_int).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Xor, expr::Value::Bool(_, lhs_bool), expr::Value::Bool(_, rhs_bool)) => {
			return Ok(expr::Value::make_bool(lhs_bool ^ rhs_bool).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Xor, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int ^ rhs_int).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Add, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int.checked_add(report, span, rhs_int)?).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Sub, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int.checked_sub(report, span, rhs_int)?).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Mul, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int.checked_mul(report, span, rhs_int)?).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Div, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int.checked_div(report, span, rhs_int)?).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Mod, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int.checked_mod(report, span, rhs_int)?).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Shl, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int.checked_shl(report, span, rhs_int)?).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Shr, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_integer(lhs_int.checked_shr(report, span, rhs_int)?).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Lt, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_bool(lhs_int < rhs_int).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Le, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_bool(lhs_int <= rhs_int).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Gt, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_bool(lhs_int > rhs_int).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Ge, expr::Value::Integer(_, lhs_int), expr::Value::Integer(_, rhs_int)) => {
			return Ok(expr::Value::make_bool(lhs_int >= rhs_int).statically_known().derived_from(&lhs).derived_from(&rhs))
		}
		(expr::BinaryOp::Concat, expr::Value::Integer(_, lhs_bigint), expr::Value::Integer(_, rhs_bigint)) => {
			match (lhs_bigint.size, rhs_bigint.size)
			{
				(Some(lhs_width), Some(rhs_width)) =>
					return Ok(expr::Value::make_integer(lhs_bigint.concat((lhs_width, 0), &rhs_bigint, (rhs_width, 0))).statically_known().derived_from(&lhs).derived_from(&rhs)),
				(None, _) =>
					return Err(report.error_span("argument to concatenation with indefinite size", lhs_expr.span())),
				(_, None) =>
					return Err(report.error_span("argument to concatenation with indefinite size", rhs_expr.span()))
			}
		}
		(expr::BinaryOp::Assign, _, _) => unreachable!(),
		(expr::BinaryOp::LazyAnd, _, _) => unreachable!(),
		(expr::BinaryOp::LazyOr, _, _) => unreachable!(),
		_ => {},
	}

	Err(report.error_span(
		format!(
			"invalid argument types to operator (have {} and {})",
			lhs.type_name(),
			rhs.type_name()),
		span))
}


impl expr::Value
{
	pub fn min_size(&self) -> usize
	{
		match &self
		{
			&expr::Value::Integer(_, bigint) => bigint.min_size(),
			_ => panic!("not an integer")
		}
	}
	

	pub fn get_bit(&self, index: usize) -> bool
	{
		match self
		{
			&expr::Value::Integer(_, ref bigint) => bigint.get_bit(index),
			_ => panic!("not an integer")
		}
	}
}