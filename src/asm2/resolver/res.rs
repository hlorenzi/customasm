use crate::*;


pub fn resolve_res(
    report: &mut diagn::Report,
    ast_res: &asm2::AstDirectiveRes,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let item_ref = ast_res.item_ref.unwrap();

    let value = asm2::resolver::eval(
        report,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext2::new(),
        &ast_res.expr)?;

    let value = value.expect_usize(
        report,
        &ast_res.expr.span(),
        Some(0))?;


    let res = defs.res_directives.get_mut(item_ref);
    let prev_value = res.reserve_size.clone();
    res.reserve_size = value;


    if res.reserve_size != prev_value
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "reserve size did not converge",
                &ast_res.expr.span());
        }

        println!("  res: {:?}", res.reserve_size);
        return Ok(asm2::ResolutionState::Unresolved);
    }

    
    Ok(asm2::ResolutionState::Resolved)
}