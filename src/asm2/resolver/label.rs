use crate::*;


pub fn resolve_label(
    report: &mut diagn::Report,
    ast_symbol: &asm2::AstSymbol,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let item_ref = ast_symbol.item_ref.unwrap();

    if let asm2::AstSymbolKind::Label = ast_symbol.kind
    {
        let symbol = defs.symbols.get(item_ref);


        // Skip this symbol if already resolved
        if !symbol.value.is_unknown()
        {
            return Ok(asm2::ResolutionState::Resolved);
        }


        // In the first iteration,
        // attempt to resolve value without guessing
        if ctx.is_first_iteration
        {
            let value = asm2::resolver::get_current_address(
                report,
                &ast_symbol.decl_span,
                defs,
                ctx,
                false)?;
                

            // Store value if successfully resolved
            if !value.is_unknown()
            {
                let symbol = defs.symbols.get_mut(item_ref);
                symbol.value = value;

                return Ok(asm2::ResolutionState::Resolved);
            }
        }


        // If could not resolve with definite values,
        // attempt to resolve with guessing
        let value_guess = asm2::resolver::get_current_address(
            report,
            &ast_symbol.decl_span,
            defs,
            ctx,
            true)?;

        
        // In the final iteration, the current guess should be
        // stable with respect to the previously guessed value
        if ctx.is_last_iteration
        {
            if value_guess != symbol.value_guess
            {
                report.error_span(
                    "label address did not converge",
                    &ast_symbol.decl_span);
            }
            
            // Store the guess as the definite value
            let symbol = defs.symbols.get_mut(item_ref);
            symbol.value = value_guess;

            return Ok(asm2::ResolutionState::Resolved);
        }


        // Store the guess
        let symbol = defs.symbols.get_mut(item_ref);
        symbol.value_guess = value_guess;
        
        Ok(asm2::ResolutionState::Unresolved)
    }
    else
    {
        unreachable!()
    }
}