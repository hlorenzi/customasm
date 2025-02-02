use crate::*;


pub fn resolve_align(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_align: &asm::AstDirectiveAlign,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_align.item_ref.unwrap();

    let value = asm::resolver::eval(
        report,
        opts,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(),
        &ast_align.expr)?;

    let value = value.expect_error_or_usize(
        report,
        ast_align.expr.span())?;

    let value = {
        match value
        {
            expr::Value::Integer(_, bigint) =>
                bigint.checked_into::<usize>(
                    report,
                    ast_align.header_span)?,

            _ => 0,
        }
    };

    let align = defs.align_directives.get_mut(item_ref);
    let prev_value = align.align_size;
    align.align_size = value;


    if align.align_size != prev_value
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "alignment size did not converge",
                ast_align.expr.span());
        }

        if opts.debug_iterations
        {
            println!("align: {:?}", align.align_size);
        }
        
        return Ok(asm::ResolutionState::Unresolved);
    }

    
    if ctx.is_last_iteration
    {
        if align.align_size == 0
        {
            report.error_span(
                "invalid alignment size",
                ast_align.expr.span());

            return Err(());
        }
    }


    Ok(asm::ResolutionState::Resolved)
}