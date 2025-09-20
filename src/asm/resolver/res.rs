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

    let value = asm::resolver::eval(
        report,
        opts,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(),
        &ast_res.expr)?;

    let value = value.expect_error_or_bigint(
        report,
        ast_res.expr.span())?;

    let value = {
        match value
        {
            expr::Value::Integer(_, bigint) =>
                bigint.checked_into::<u32>(
                    report,
                    ast_res.expr.span())?,
                
            _ => 0,
        }
    };

    let bank = defs.bankdefs.get(ctx.bank_ref);
    let res = defs.res_directives.get_mut(item_ref);
    let prev_value = res.reserve_size;
    
    res.reserve_size =
        <u32 as TryInto<usize>>::try_into(value).unwrap() *
        bank.addr_unit;


    if res.reserve_size != prev_value
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "reserve size did not converge",
                ast_res.expr.span());
        }

        if opts.debug_iterations
        {
            println!("  res: {:?}", res.reserve_size);
        }
        
        return Ok(asm::ResolutionState::Unresolved);
    }

    
    Ok(asm::ResolutionState::Resolved)
}