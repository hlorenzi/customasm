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

    if let asm::AstSymbolKind::Label = ast_symbol.kind
    {
        let value = ctx.eval_address(
            report,
            ast_symbol.decl_span,
            defs,
            ctx.can_guess())?;
        

        let symbol = defs.symbols.get_mut(item_ref);
        let prev_value = symbol.value.clone();
        symbol.value = expr::Value::Integer(value);
        symbol.bankdef_ref = Some(ctx.bank_ref);


        if symbol.value != prev_value
        {
            // On the final iteration, unstable guesses become errors
            if ctx.is_last_iteration
            {
                report.error_span(
                    "label address did not converge",
                    ast_symbol.decl_span);
            }
            
            if opts.debug_iterations
            {
                println!("label: {} = {:?}",
                    ast_symbol.name,
                    symbol.value);
            }
            
            return Ok(asm::ResolutionState::Unresolved);
        }


        Ok(asm::ResolutionState::Resolved)
    }
    else
    {
        unreachable!()
    }
}