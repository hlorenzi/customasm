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

    let align = defs.align_directives.get(item_ref);

    if align.resolved && opts.optimize_statically_known {
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
        &ast_align.expr)?;

    let align = defs.align_directives.get_mut(item_ref);
    let is_stable = value.is_stable(&align.value);
    align.value = value;
    
    let align_size = finalize_align(
        report,
        ast_align,
        defs,
        ctx)?;

    let align = defs.align_directives.get_mut(item_ref);
    align.align_size = align_size;

    asm::resolver::handle_value_resolution(
        opts,
        report,
        ast_align.expr.span(),
        ctx.can_guess(),
        align.value.is_guess(),
        is_stable,
        &mut align.resolved,
        false,
        "align",
        "alignment value",
        None,
        &align.value)
}


pub fn finalize_align(
    report: &mut diagn::Report,
    ast_align: &asm::AstDirectiveAlign,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<usize, ()>
{
    let item_ref = ast_align.item_ref.unwrap();

    let align = defs.align_directives.get(item_ref);

    if let expr::Value::Integer(_, bigint) = &align.value
    {
        let align_size = bigint.maybe_into::<usize>();

        if let Some(align_size) = align_size
        {
            if align_size == 0
            {
                if ctx.can_guess() {
                    return Ok(0);
                }

                report.error_span(
                    "invalid alignment size",
                    ast_align.expr.span());

                return Err(());
            }
            
            return Ok(align_size);
        }
        
        if ctx.can_guess() {
            return Ok(0);
        }
        
        report.error_span(
            "alignment size outside the supported range",
            ast_align.expr.span());
    }
    else
    {
        if ctx.can_guess() {
            return Ok(0);
        }

        report.error_span(
            format!(
                "invalid type for alignment size (have {})",
                align.value.type_name()),
            ast_align.expr.span());
    }

    Err(())
}