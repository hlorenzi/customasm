use crate::*;


pub fn resolve_label(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast_symbol: &asm::AstSymbol,
    _decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_symbol.item_ref.unwrap();

    let symbol = defs.symbols.get(item_ref);
    if symbol.resolved &&
        symbol.value_statically_known
    {
        return Ok(asm::ResolutionState::Resolved);
    }

    let asm::AstSymbolKind::Label = ast_symbol.kind
        else { unreachable!() };
        
    let value = ctx.eval_address(
        report,
        ast_symbol.decl_span,
        defs,
        ctx.can_guess())?;
    

    let symbol = defs.symbols.get_mut(item_ref);
    let prev_value = symbol.value.clone();
    symbol.value = expr::Value::make_integer(value);
    symbol.value.get_mut_metadata().symbol_ref = Some(item_ref);
    symbol.bankdef_ref = Some(ctx.bank_ref);

    if opts.debug_iterations
    {
        println!("label: {} = {:?}",
            ast_symbol.name,
            symbol.value);
    }
    
    symbol.resolved =
        symbol.value == prev_value &&
        !symbol.value.is_unknown();
    
    if !symbol.resolved
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "label address did not converge",
                ast_symbol.decl_span);
        }
        
        return Ok(asm::ResolutionState::Unresolved);
    }

    Ok(asm::ResolutionState::Resolved)
}