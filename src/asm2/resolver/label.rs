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
        let value = asm2::resolver::get_current_address(
            report,
            &ast_symbol.decl_span,
            defs,
            ctx,
            true)?;
                

        let symbol = defs.symbols.get_mut(item_ref);
        let prev_value = symbol.value.clone();
        symbol.value = value;


        if symbol.value != prev_value
        {
            // On the final iteration, unstable guesses become errors
            if ctx.is_last_iteration
            {
                report.error_span(
                    "label address did not converge",
                    &ast_symbol.decl_span);
            }
            
            println!("label: {} = {:?}",
                ast_symbol.name,
                symbol.value);
            return Ok(asm2::ResolutionState::Unresolved);
        }


        Ok(asm2::ResolutionState::Resolved)
    }
    else
    {
        unreachable!()
    }
}