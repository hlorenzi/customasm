use crate::*;


pub fn eval(
    report: &mut diagn::Report,
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
            if info.hierarchy == &["$"]
            {
                return get_current_address(
                    info.report,
                    info.span,
                    defs,
                    ctx);
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

    let mut provider = expr::EvalProvider {
        eval_var: &mut eval_var,
        eval_fn: &mut expr::dummy_eval_fn(),
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
            if info.hierarchy == &["$"]
            {
                info.report.error_span(
                    "cannot use `$` in this context",
                    info.span);
        
                return Err(());
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
	
	
pub fn get_current_address(
    report: &mut diagn::Report,
    span: &diagn::Span,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<expr::Value, ()>
{
    let bankdef = &defs.bankdefs.get(ctx.bank_ref);
    let addr_unit = bankdef.addr_unit;

    let cur_position = ctx.bank_data.cur_position;
    
    // FIXME: force non-guess on last iteration
    let excess_bits = cur_position % addr_unit;
    if excess_bits != 0 && !ctx.can_guess()
    {
        let bits_short = addr_unit - excess_bits;

        let plural = {
            if bits_short > 1
                { "bits" }
            else
                { "bit" }
        };

        report.error_span(
            format!(
                "position is not aligned to an address ({} {} to next)",
                bits_short, plural),
            span);

        return Err(());
    }
        
    let addr = expr::Value::make_integer(
        &util::BigInt::from(cur_position / addr_unit) +
            &bankdef.addr_start);
    
    Ok(addr)
}