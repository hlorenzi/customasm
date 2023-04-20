use crate::*;


pub fn resolve_res(
    report: &mut diagn::Report,
    opts: &asm2::AssemblyOptions,
    fileserver: &dyn util::FileServer,
    ast_res: &asm2::AstDirectiveRes,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let item_ref = ast_res.item_ref.unwrap();

    let value = asm2::resolver::eval(
        report,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext2::new(),
        &ast_res.expr)?;

    let value = value.expect_error_or_usize(
        report,
        &ast_res.expr.span())?;

    let value = {
        match value
        {
            expr::Value::Integer(bigint) =>
                bigint.checked_to_usize().unwrap(),
                
            _ => 0,
        }
    };

    let bank = defs.bankdefs.get(ctx.bank_ref);
    let res = defs.res_directives.get_mut(item_ref);
    let prev_value = res.reserve_size.clone();
    // FIXME: Multiplication can overflow
    res.reserve_size = value * bank.addr_unit;


    if res.reserve_size != prev_value
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "reserve size did not converge",
                &ast_res.expr.span());
        }

        if opts.debug_iterations
        {
            println!("  res: {:?}", res.reserve_size);
        }
        
        return Ok(asm2::ResolutionState::Unresolved);
    }

    
    Ok(asm2::ResolutionState::Resolved)
}