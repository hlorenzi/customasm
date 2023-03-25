use crate::*;


/// Tries to resolve the value of constants as much
/// as possible, for whatever number of iterations it takes.
/// 
/// Stops as soon as one iteration reports having resolved
/// no new constants.
pub fn resolve_constants(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    let mut prev_resolved_count = 0;

    loop
    {
        let resolved_count = resolve_constants_once(
            report,
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
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<usize, ()>
{
    let mut resolved_count = 0;

    let mut iter = asm2::ResolveIterator::new(
        ast,
        defs,
        true,
        false);

    while let Some(ctx) = iter.next(decls, defs)
    {
        if let asm2::AstAny::Symbol(ast_symbol) = ctx.node
        {
            if let asm2::AstSymbolKind::Constant(_) = ast_symbol.kind
            {
                let resolution_state = resolve_constant(
                    report,
                    ast_symbol,
                    decls,
                    defs,
                    &ctx)?;

                if let asm2::ResolutionState::Resolved = resolution_state
                {
                    resolved_count += 1;
                }
            }
        }

        iter.update_after_node(decls, defs);
    }

    Ok(resolved_count)
}


pub fn resolve_constant(
    report: &mut diagn::Report,
    ast_symbol: &asm2::AstSymbol,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let item_ref = ast_symbol.item_ref.unwrap();

    if let asm2::AstSymbolKind::Constant(ref ast_const) = ast_symbol.kind
    {
        let value = asm2::resolver::eval(
            report,
            decls,
            defs,
            ctx,
            &mut expr::EvalContext2::new(),
            &ast_const.expr)?;


        let symbol = defs.symbols.get_mut(item_ref);
        let prev_value = symbol.value.clone();
        symbol.value = value;


        if symbol.value != prev_value
        {
            // On the final iteration, unstable guesses become errors
            if ctx.is_last_iteration
            {
                report.error_span(
                    "constant value did not converge",
                    &ast_symbol.decl_span);
            }

            println!("const: {} = {:?}",
                ast_symbol.name,
                symbol.value);
            return Ok(asm2::ResolutionState::Unresolved);
        }

        
        return Ok(asm2::ResolutionState::Resolved);
    }
    else
    {
        unreachable!()
    }
}