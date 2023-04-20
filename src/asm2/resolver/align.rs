use crate::*;


pub fn resolve_align(
    report: &mut diagn::Report,
    opts: &asm2::AssemblyOptions,
    fileserver: &dyn util::FileServer,
    ast_align: &asm2::AstDirectiveAlign,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let item_ref = ast_align.item_ref.unwrap();

    let value = asm2::resolver::eval(
        report,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext2::new(),
        &ast_align.expr)?;

    let value = value.expect_error_or_usize(
        report,
        &ast_align.expr.span())?;

    let value = {
        match value
        {
            expr::Value::Integer(bigint) =>
                bigint.checked_to_usize().unwrap(),

            _ => 0,
        }
    };

    let align = defs.align_directives.get_mut(item_ref);
    let prev_value = align.align_size.clone();
    align.align_size = value;


    if align.align_size != prev_value
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "alignment size did not converge",
                &ast_align.expr.span());
        }

        if opts.debug_iterations
        {
            println!("align: {:?}", align.align_size);
        }
        
        return Ok(asm2::ResolutionState::Unresolved);
    }

    
    if ctx.is_last_iteration
    {
        if align.align_size == 0
        {
            report.error_span(
                "invalid alignment size",
                &ast_align.expr.span());

            return Err(());
        }
    }


    Ok(asm2::ResolutionState::Resolved)
}