use crate::*;


pub fn eval(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    eval_ctx: &mut expr::EvalContext,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut provider = |query: expr::EvalQuery|
    {
        match query
        {
            expr::EvalQuery::CtxLabel(query_ctxlabel) =>
                asm::resolver::eval_ctxlabel(
                    decls,
                    defs,
                    ctx,
                    query_ctxlabel),
                    
            expr::EvalQuery::Variable(query_var) =>
                asm::resolver::eval_variable(
                    decls,
                    defs,
                    ctx,
                    query_var),
                    
            expr::EvalQuery::Member(query_member) =>
                asm::resolver::eval_member(
                    decls,
                    defs,
                    Some(ctx),
                    query_member),
                    
            expr::EvalQuery::Function(query_fn) =>
                asm::resolver::eval_fn(
                    opts,
                    fileserver,
                    decls,
                    defs,
                    ctx,
                    query_fn),
                
            expr::EvalQuery::AsmBlock(query_asm) =>
                asm::resolver::eval_asm(
                    opts,
                    fileserver,
                    decls,
                    defs,
                    ctx,
                    query_asm),
        }
    };

    expr.eval_with_ctx(
        report,
        eval_ctx,
        &mut provider)
}


/// Evaluates an expression without relying on
/// addresses, banks, user-defined functions,
/// or user-defined instructions.
pub fn eval_simple(
    report: &mut diagn::Report,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut provider = |query: expr::EvalQuery|
    {
        match query
        {
            expr::EvalQuery::CtxLabel(_) =>
                Ok(expr::Value::make_unknown()),
                
            expr::EvalQuery::Variable(query_var) =>
                asm::resolver::eval_variable_simple(
                    decls,
                    defs,
                    query_var),
                    
            expr::EvalQuery::Member(query_member) =>
                asm::resolver::eval_member(
                    decls,
                    defs,
                    None,
                    query_member),
                    
            expr::EvalQuery::Function(_) =>
                Ok(expr::Value::make_unknown()),
                
            expr::EvalQuery::AsmBlock(_) =>
                Ok(expr::Value::make_unknown()),
        }
    };

    let result = expr.eval_with_ctx(
        report,
        &mut expr::EvalContext::new(),
        &mut provider)?;

    match result
    {
        expr::Value::FailedConstraint(_, msg) =>
        {
            report.message(msg);
            Err(())
        }

        _ => Ok(result)
    }
}


pub fn eval_certain(
    report: &mut diagn::Report,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut provider = |query: expr::EvalQuery|
    {
        match query
        {
            expr::EvalQuery::CtxLabel(_) =>
                Ok(expr::Value::make_unknown()),
                
            expr::EvalQuery::Variable(query_var) =>
                asm::resolver::eval_variable_certain(
                    decls,
                    defs,
                    query_var),
                    
            expr::EvalQuery::Member(query_member) =>
                asm::resolver::eval_member(
                    decls,
                    defs,
                    None,
                    query_member),
                    
            expr::EvalQuery::Function(_) =>
                Ok(expr::Value::make_unknown()),
                
            expr::EvalQuery::AsmBlock(_) =>
                Ok(expr::Value::make_unknown()),
        }
    };

    let result = expr.eval_with_ctx(
        report,
        &mut expr::EvalContext::new(),
        &mut provider)?;

    match result
    {
        expr::Value::Unknown(_) =>
        {
            report.error_span(
                "cannot resolve expression",
                expr.span());
    
            Err(())
        }

        expr::Value::FailedConstraint(_, msg) =>
        {
            report.message(msg);
            Err(())
        }

        _ => Ok(result)
    }
}


pub fn eval_ctxlabel(
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalCtxLabelQuery)
    -> Result<expr::Value, ()>
{
    let symbol_ref = decls.symbols.get_current_label(
        query.report,
        query.span,
        ctx.symbol_ctx,
        query.nesting_level)?;

    let symbol = defs.symbols.get(symbol_ref);

    if symbol.value.is_unknown() &&
        !ctx.can_guess()
    {
        query.report.error_span(
            format!(
                "unresolved symbol `{}`",
                decls.symbols.get_displayable_name(
                    0,
                    ctx.symbol_ctx.get_hierarchy())),
            query.span);

        return Err(());
    }

    Ok(symbol.value.clone())
}


pub fn eval_variable(
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalVariableQuery)
    -> Result<expr::Value, ()>
{
    if query.hierarchy_level == 0
    {
        let maybe_builtin = eval_builtin_symbol(
            decls,
            defs,
            ctx,
            query,
            query.hierarchy[0].as_ref())?;

        if let Some(builtin) = maybe_builtin
        {
            return Ok(builtin);
        }
    }

    let symbol_ref = decls.symbols.get_by_name(
        query.report,
        query.span,
        ctx.symbol_ctx,
        query.hierarchy_level,
        query.hierarchy)?;

    let symbol = defs.symbols.get(symbol_ref);

    if symbol.value.is_unknown() &&
        !ctx.can_guess()
    {
        query.report.error_span(
            format!(
                "unresolved symbol `{}`",
                decls.symbols.get_displayable_name(
                    query.hierarchy_level,
                    query.hierarchy)),
            query.span);

        return Err(());
    }

    Ok(symbol.value.clone())
}


/// Evaluates a variable/symbol without relying on
/// addresses or banks.
pub fn eval_variable_simple(
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    query: &mut expr::EvalVariableQuery)
    -> Result<expr::Value, ()>
{
    if query.hierarchy_level == 0
    {
        match query.hierarchy[0].as_ref()
        {
            "$" | "pc" => return Ok(expr::Value::make_unknown()),
            _ => {}
        }
    }

    let symbol_ref = decls.symbols.try_get_by_name(
        &util::SymbolContext::new_global(),
        query.hierarchy_level,
        query.hierarchy);

    match symbol_ref
        .map(|s| defs.symbols.maybe_get(s))
        .flatten()
    {
        Some(symbol) => Ok(symbol.value.clone()),
        None => Ok(expr::Value::make_unknown()),
    }
}


pub fn eval_variable_certain(
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    query: &mut expr::EvalVariableQuery)
    -> Result<expr::Value, ()>
{
    if query.hierarchy_level == 0
    {
        match query.hierarchy[0].as_ref()
        {
            "$" | "pc" =>
            {
                query.report.error_span(
                    "cannot get address in this context",
                    query.span);
        
                return Err(());
            }

            _ => {}
        }
    }

    let symbol_ref = decls.symbols.get_by_name(
        query.report,
        query.span,
        &util::SymbolContext::new_global(),
        query.hierarchy_level,
        query.hierarchy)?;

    let value = defs.symbols
        .maybe_get(symbol_ref)
        .map_or(expr::Value::make_unknown(), |s| s.value.clone());

    if value.is_unknown()
    {
        query.report.error_span(
            format!(
                "unresolved symbol `{}`",
                decls.symbols.get_displayable_name(
                    query.hierarchy_level,
                    query.hierarchy)),
            query.span);

        return Err(());
    }

    Ok(value)
}


fn eval_builtin_symbol(
    _decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalVariableQuery,
    name: &str)
    -> Result<Option<expr::Value>, ()>
{
    match name
    {
        "$" | "pc" =>
        {
            let addr = ctx.eval_address(
                query.report,
                query.span,
                defs,
                ctx.can_guess())?;
            
            Ok(Some(expr::Value::make_integer(addr)
                .with_bank_ref(ctx.bank_ref)))
        }

        _ =>
        {
            if let Some(_) = asm::resolver::resolve_builtin_fn(name)
            {
                Ok(Some(expr::Value::AsmBuiltInFunction(
                    expr::Value::make_metadata(),
                    name.to_string())))
            }
            else
            {
                Ok(None)
            }
        }
    }
}


pub fn eval_member(
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    _ctx: Option<&asm::ResolverContext>,
    query: &mut expr::EvalMemberQuery)
    -> Result<expr::Value, ()>
{
    if let Some(item_ref) = query.value.get_metadata().symbol_ref
    {
        decls.symbols.get(item_ref);

        let subsymbol_ref = decls.symbols.traverse(
            Some(item_ref),
            &[query.member_name]);

        if let Some(subsymbol_ref) = subsymbol_ref
        {
            let subsymbol = defs.symbols.get(subsymbol_ref);
            return Ok(subsymbol.value.clone());
        }
    }

    if let Some(value) = eval_member_bankdef(decls, defs, _ctx, query)?
    {
        return Ok(value);
    }
    
	if let Some(value) = expr::resolve_builtin_member(query)?
	{
		return Ok(value);
	}

    query.report.error_span(
        format!(
            "unknown symbol `{}`",
            query.member_name),
        query.span);

    Err(())
}


pub fn eval_member_bankdef(
    _decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    _ctx: Option<&asm::ResolverContext>,
    query: &mut expr::EvalMemberQuery)
    -> Result<Option<expr::Value>, ()>
{
    let expr::Value::Bankdef(_, bank_ref) = query.value
        else { return Ok(None); };

    let bank = defs.bankdefs.get(bank_ref);

    match query.member_name
    {
        "bits" => Ok(Some(expr::Value::make_integer(bank.addr_unit))),
        "addr" => Ok(Some(expr::Value::make_integer(bank.addr_start.clone()))),
        "outp" => Ok(Some(expr::Value::make_maybe_integer(bank.output_offset))),
        "size" => Ok(Some(expr::Value::make_maybe_integer(bank.size_in_units))),
        "size_b" => Ok(Some(expr::Value::make_maybe_integer(bank.size_in_bits))),
        "data" => Ok(Some(bank.userdata.clone())),
        _ => Ok(None)
    }
}