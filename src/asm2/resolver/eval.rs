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
        eval_builtin_fn(
            fileserver,
            decls,
            defs,
            ctx,
            info)
    };

    let mut provider = expr::EvalProvider {
        eval_var: &mut eval_var,
        eval_fn: &mut eval_fn,
        eval_asm: &mut expr::dummy_eval_asm(),
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
            if let Some(_) = resolve_builtin_fn(name)
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


fn eval_builtin_fn(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    if let expr::Value::AsmBuiltInFunction(ref name) = info.func
    {
        let builtin_fn = resolve_builtin_fn(name).unwrap();

        builtin_fn(
            fileserver,
            decls,
            defs,
            ctx,
            info)
    }
    else
    {
        unreachable!()
    }
}



type BuiltinFn = fn(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>;


fn resolve_builtin_fn(
    name: &str)
    -> Option<BuiltinFn>
{
    match name
    {
        "incbin" => Some(eval_builtin_incbin),
        "incbinstr" => Some(eval_builtin_incbinstr),
        "inchexstr" => Some(eval_builtin_inchexstr),
        _ => None,
    }
}


fn eval_builtin_incbin(
    fileserver: &dyn util::FileServer,
    _decls: &asm2::ItemDecls,
    _defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let relative_filename = info.args[0].value.expect_string(
        info.report,
        info.args[0].span)?;

    let absolute_filename = util::filename_navigate2(
        info.report,
        &ctx.filename_ctx.unwrap(),
        &relative_filename.utf8_contents,
        &info.span)?;

    let bytes = fileserver.get_bytes2(
        info.report,
        Some(&info.span),
        &absolute_filename)?;

    Ok(expr::Value::make_integer(
        util::BigInt::from_bytes_be(&bytes)))
}


fn eval_builtin_incbinstr(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    eval_builtin_incstr(
        fileserver,
        1,
        decls,
        defs,
        ctx,
        info)
}


fn eval_builtin_inchexstr(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    eval_builtin_incstr(
        fileserver,
        4,
        decls,
        defs,
        ctx,
        info)
}


fn eval_builtin_incstr(
    fileserver: &dyn util::FileServer,
    bits_per_char: usize,
    _decls: &asm2::ItemDecls,
    _defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let relative_filename = info.args[0].value.expect_string(
        info.report,
        info.args[0].span)?;

    let absolute_filename = util::filename_navigate2(
        info.report,
        &ctx.filename_ctx.unwrap(),
        &relative_filename.utf8_contents,
        &info.span)?;

    let chars = fileserver.get_chars2(
        info.report,
        Some(&info.span),
        &absolute_filename)?;

    
    let mut bitvec = util::BitVec::new();

    for c in chars
    {
        if syntax::is_whitespace(c) ||
            c == '_' ||
            c == '\r' || c == '\n'
        {
            continue;
        }

        let digit = match c.to_digit(1 << bits_per_char)
        {
            Some(digit) => digit,
            None =>
            {
                info.report.error_span(
                    "invalid character in file contents",
                    &info.span);
                
                return Err(());
            }
        };
        
        for i in 0..bits_per_char
        {
            let bit = (digit & (1 << (bits_per_char - 1 - i))) != 0;
            bitvec.write(bitvec.len(), bit);
        }
    }

    // TODO: Optimize conversion to bigint
    Ok(expr::Value::make_integer(bitvec.as_bigint()))
}