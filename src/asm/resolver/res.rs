use crate::*;


pub fn resolve_res(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_res: &asm::AstDirectiveRes,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_res.item_ref.unwrap();

    let res = defs.res_directives.get(item_ref);

    if res.resolved && opts.optimize_statically_known {
        return Ok(asm::ResolutionState::Resolved);
    }

    let value = asm::resolver::eval(
        report,
        fileserver,
        opts,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(opts),
        &ast_res.expr)?;

    let res = defs.res_directives.get_mut(item_ref);
    let is_stable = value.is_stable(&res.value);
    res.value = value;
    
    let reserve_size = finalize_res(
        report,
        ast_res,
        defs,
        ctx)?;
        
    let res = defs.res_directives.get_mut(item_ref);
    res.reserve_size = reserve_size;

    asm::resolver::handle_value_resolution(
        opts,
        report,
        ast_res.expr.span(),
        ctx.can_guess(),
        res.value.is_guess(),
        is_stable,
        &mut res.resolved,
        false,
        "res",
        "reserve size",
        None,
        &res.value)
}


pub fn finalize_res(
    report: &mut diagn::Report,
    ast_res: &asm::AstDirectiveRes,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<usize, ()>
{
    let item_ref = ast_res.item_ref.unwrap();

    let res = defs.res_directives.get(item_ref);

    if let expr::Value::Integer(_, bigint) = &res.value
    {
        if let Some(reserve_units) = bigint.maybe_into::<usize>()
        {
            let bank = defs.bankdefs.get(ctx.bank_ref);
            if let Some(reserve_bits) = reserve_units.checked_mul(bank.addr_unit)
            {
                return Ok(reserve_bits);
            }
        }

        if ctx.can_guess() {
            return Ok(0);
        }

        report.error_span(
            "reserve size outside supported range",
            ast_res.expr.span());
    }
    else
    {
        if ctx.can_guess() {
            return Ok(0);
        }
        
        report.error_span(
            format!(
                "invalid type for reserve size (have {})",
                res.value.type_name()),
            ast_res.expr.span());
    }

    Err(())
}