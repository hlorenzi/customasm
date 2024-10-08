use crate::*;


pub fn resolve_assert(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_assert: &asm::AstDirectiveAssert,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    if !ctx.is_last_iteration
    {
        return Ok(asm::ResolutionState::Unresolved);
    }
    
    let value = asm::resolver::eval(
        report,
        opts,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(),
        &ast_assert.condition_expr)?;

    let satisfied = value.expect_bool(
        report,
        ast_assert.condition_expr.span())?;

    if !satisfied
    {
        report.error_span(
            "assertion failed",
            ast_assert.condition_expr.span());
    }
    
    Ok(asm::ResolutionState::Resolved)
}