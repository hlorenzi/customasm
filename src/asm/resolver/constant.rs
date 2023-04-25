use crate::*;


/// Tries to resolve the value of constants as much
/// as possible, for whatever number of iterations it takes.
/// 
/// Stops as soon as one iteration reports having resolved
/// no new constants.
pub fn resolve_constants(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &dyn util::FileServer,
    ast: &asm::AstTopLevel,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    let mut prev_resolved_count = 0;

    loop
    {
        let resolved_count = resolve_constants_once(
            report,
            opts,
            fileserver,
            ast,
            decls,
            defs)?;

        if resolved_count == prev_resolved_count
        {
            return Ok(());
        }

        prev_resolved_count = resolved_count;
    }
}


pub fn resolve_constants_once(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &dyn util::FileServer,
    ast: &asm::AstTopLevel,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<usize, ()>
{
    let mut resolved_count = 0;

    let mut iter = asm::ResolveIterator::new(
        ast,
        defs,
        true,
        false);

    while let Some(ctx) = iter.next(decls, defs)
    {
        if let asm::ResolverNode::Symbol(ast_symbol) = ctx.node
        {
            if let asm::AstSymbolKind::Constant(_) = ast_symbol.kind
            {
                let resolution_state = resolve_constant(
                    report,
                    opts,
                    fileserver,
                    ast_symbol,
                    decls,
                    defs,
                    &ctx)?;

                if let asm::ResolutionState::Resolved = resolution_state
                {
                    resolved_count += 1;
                }
            }
        }
    }

    Ok(resolved_count)
}


pub fn resolve_constant(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &dyn util::FileServer,
    ast_symbol: &asm::AstSymbol,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_symbol.item_ref.unwrap();

    if let asm::AstSymbolKind::Constant(ref ast_const) = ast_symbol.kind
    {
        let value = asm::resolver::eval(
            report,
            fileserver,
            decls,
            defs,
            ctx,
            &mut expr::EvalContext::new(),
            &ast_const.expr)?;


        let symbol = defs.symbols.get_mut(item_ref);
        let prev_value = symbol.value.clone();
        symbol.value = value;

        
        // Optimize future iterations for the case where it's
        // statically known that the encoding can be resolved
        // in the first pass
        if opts.optimize_statically_known &&
            ctx.is_first_iteration &&
            symbol.value_statically_known
        {
            if opts.debug_iterations
            {
                println!("const: {} = {:?} [static]",
                    ast_symbol.name,
                    symbol.value);
            }

            symbol.resolved = true;
            return Ok(asm::ResolutionState::Resolved);
        }


        if symbol.value != prev_value
        {
            // On the final iteration, unstable guesses become errors
            if ctx.is_last_iteration
            {
                report.error_span(
                    "constant value did not converge",
                    &ast_symbol.decl_span);
            }

            if opts.debug_iterations
            {
                println!("const: {} = {:?}",
                    ast_symbol.name,
                    symbol.value);
            }

            return Ok(asm::ResolutionState::Unresolved);
        }

        
        return Ok(asm::ResolutionState::Resolved);
    }
    else
    {
        unreachable!()
    }
}