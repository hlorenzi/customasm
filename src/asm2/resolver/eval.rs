use crate::*;


pub fn eval(
    report: &mut diagn::Report,
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    eval_ctx: &mut expr::EvalContext2,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut eval_var = |info: &mut expr::EvalVariableInfo2|
    {
        if info.hierarchy_level == 0
        {
            let maybe_builtin = eval_builtin_symbol(
                decls,
                defs,
                ctx,
                info,
                info.hierarchy[0].as_ref())?;

            if let Some(builtin) = maybe_builtin
            {
                return Ok(builtin);
            }
        }

        let symbol_ref = decls.symbols.get_by_name(
            info.report,
            info.span,
            ctx.symbol_ctx,
            info.hierarchy_level,
            info.hierarchy)?;

        let symbol = defs.symbols.get(symbol_ref);

        let value = {
            match symbol.value.clone()
            {
                value @ expr::Value::Unknown =>
                {
                    if !ctx.can_guess()
                    {
                        info.report.error_span(
                            format!(
                                "unresolved symbol `{}`",
                                decls.symbols.get_displayable_name(
                                    info.hierarchy_level,
                                    info.hierarchy)),
                            info.span);
                
                        return Err(());
                    }

                    value
                }

                value => value,
            }
        };

        Ok(value)
    };

    let mut eval_fn = |info: &mut expr::EvalFunctionInfo2|
    {
        asm2::resolver::eval_builtin_fn(
            fileserver,
            decls,
            defs,
            ctx,
            info)
    };

    let mut eval_asm = |info: &mut expr::EvalAsmInfo2|
    {
        asm2::resolver::eval_asm(
            fileserver,
            decls,
            defs,
            ctx,
            info)
    };

    let mut provider = expr::EvalProvider {
        eval_var: &mut eval_var,
        eval_fn: &mut eval_fn,
        eval_asm: &mut eval_asm,
    };

    expr.eval2_with_ctx(
        report,
        eval_ctx,
        &mut provider)
}


pub fn eval_simple(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut eval_var = |info: &mut expr::EvalVariableInfo2|
    {
        if info.hierarchy_level == 0
        {
            match info.hierarchy[0].as_ref()
            {
                "$" | "pc" =>
                {
                    info.report.error_span(
                        "cannot get address in this context",
                        info.span);
            
                    return Err(());
                }

                _ => {}
            }
        }

        let symbol_ref = decls.symbols.get_by_name(
            info.report,
            info.span,
            &util::SymbolContext::new_global(),
            info.hierarchy_level,
            info.hierarchy)?;

        let symbol = defs.symbols.get(symbol_ref);

        let value = {
            match symbol.value.clone()
            {
                expr::Value::Unknown =>
                {
                    info.report.error_span(
                        format!(
                            "unresolved symbol `{}`",
                            decls.symbols.get_displayable_name(
                                info.hierarchy_level,
                                info.hierarchy)),
                        info.span);
            
                    return Err(());
                }

                value => value,
            }
        };

        Ok(value)
    };

    let mut provider = expr::EvalProvider {
        eval_var: &mut eval_var,
        eval_fn: &mut expr::dummy_eval_fn(),
        eval_asm: &mut expr::dummy_eval_asm(),
    };

    let result = expr.eval2_with_ctx(
        report,
        &mut expr::EvalContext2::new(),
        &mut provider)?;

    match result
    {
        expr::Value::Unknown =>
        {
            report.error_span(
                "cannot resolve expression",
                &expr.span());
    
            Err(())
        }

        expr::Value::FailedConstraint(msg) =>
        {
            report.message(msg);
            Err(())
        }

        _ => Ok(result)
    }
}


fn eval_builtin_symbol(
    _decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalVariableInfo2,
    name: &str)
    -> Result<Option<expr::Value>, ()>
{
    match name
    {
        "$" | "pc" =>
        {
            Ok(Some(expr::Value::Integer(ctx.eval_address(
                info.report,
                info.span,
                defs,
                ctx.can_guess())?)))
        }

        _ =>
        {
            if let Some(_) = asm2::resolver::resolve_builtin_fn(name)
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