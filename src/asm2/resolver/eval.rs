use crate::*;


pub fn eval(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    eval_ctx: &mut expr::EvalContext2,
    can_guess: bool,
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
                    ctx,
                    can_guess);
            }
        }

        let symbol_ref = decls.symbols.get_by_name(
            info.report,
            info.span,
            ctx.symbol_ctx,
            info.hierarchy_level,
            info.hierarchy)?;

        let symbol = defs.symbols.get(symbol_ref);

        let value = symbol.value.clone();
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
	
	
pub fn get_current_address(
    report: &mut diagn::Report,
    span: &diagn::Span,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    can_guess: bool)
    -> Result<expr::Value, ()>
{
    let bankdef = &defs.bankdefs.get(ctx.bank_ref);
    let addr_unit = bankdef.addr_unit;

    let cur_position = ctx.bank_data.cur_position;
    
    let excess_bits = cur_position % addr_unit;
    if excess_bits != 0 && !can_guess
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
                "position is not aligned to an address boundary ({} {} short)",
                bits_short, plural),
            span);

        return Err(());
    }
        
    let addr = expr::Value::make_integer(
        &util::BigInt::from(cur_position / addr_unit) +
            &bankdef.addr_start);
    
    Ok(addr)
}