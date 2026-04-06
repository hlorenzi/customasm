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
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(opts),
        &ast_align.expr)?;

    let align_size = {
        match value
        {
            expr::Value::Integer(_, ref bigint) =>
                bigint.maybe_into::<usize>().unwrap_or(0),

            _ => 0,
        }
    };

    let align = defs.align_directives.get_mut(item_ref);
    let is_stable = value.is_stable(&align.value);
    align.align_size = align_size;
    align.value = value;
    
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


pub fn check_final_align(
    report: &mut diagn::Report,
    ast_align: &asm::AstDirectiveAlign,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    let item_ref = ast_align.item_ref.unwrap();

    let align = defs.align_directives.get(item_ref);

    if let expr::Value::Integer(_, bigint) = &align.value
    {
        bigint.checked_into::<usize>(report, ast_align.expr.span())?;
        
        if align.align_size == 0
        {
            report.error_span(
                "invalid alignment size",
                ast_align.expr.span());

            return Err(());
        }
    }
    else
    {
        report.error_span(
            format!(
                "invalid type for data element (have {})",
                align.value.type_name()),
            ast_align.expr.span());

        return Err(());
    }

    Ok(())
}