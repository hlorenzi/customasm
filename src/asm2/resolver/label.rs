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

        if !symbol.value.is_unknown()
        {
            return Ok(asm2::ResolutionState::Resolved);
        }


        let value = asm2::resolver::get_current_address(
            report,
            &ast_symbol.decl_span,
            defs,
            ctx,
            false)?;
            

        if value.is_unknown()
        {
            if ctx.is_final_iteration
            {
                report.error_span(
                    "label address did not converge",
                    &ast_symbol.decl_span);
            }

            let value_guess = asm2::resolver::get_current_address(
                report,
                &ast_symbol.decl_span,
                defs,
                ctx,
                true)?;

            let symbol = defs.symbols.get_mut(item_ref);
            symbol.value_guess = Some(value_guess);
            
            println!("{:#?}", defs.symbols);

            Ok(asm2::ResolutionState::Unresolved)
        }
        else
        {
            let symbol = defs.symbols.get_mut(item_ref);
            symbol.value = value;

            Ok(asm2::ResolutionState::Resolved)
        }
    }
    else
    {
        unreachable!()
    }
}