use crate::*;


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

    let value = {
        match symbol.value.clone()
        {
            value @ expr::Value::Unknown =>
            {
                if !ctx.can_guess()
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

                value
            }

            value => value,
        }
    };

    Ok(value)
}


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

    let symbol = defs.symbols.get(symbol_ref);

    let value = {
        match symbol.value.clone()
        {
            expr::Value::Unknown =>
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

            value => value,
        }
    };

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
            Ok(Some(expr::Value::Integer(ctx.eval_address(
                query.report,
                query.span,
                defs,
                ctx.can_guess())?)))
        }

        _ =>
        {
            if let Some(_) = asm::resolver::resolve_builtin_fn(name)
            {
                Ok(Some(expr::Value::AsmBuiltInFunction(name.to_string())))
            }
            else
            {
                Ok(None)
            }
        }
    }
}