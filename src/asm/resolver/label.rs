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

    if symbol.resolved {
        return Ok(asm::ResolutionState::Resolved);
    }

    let asm::AstSymbolKind::Label = ast_symbol.kind
        else { unreachable!() };
        
    let value = expr::Value::make_integer(
        ctx.eval_address(
            report,
            ast_symbol.decl_span,
            defs,
            ctx.can_guess())?);

    let symbol = defs.symbols.get_mut(item_ref);
    let is_stable = value.is_stable(&symbol.value);
    symbol.value = value;
    symbol.value.get_mut_metadata().symbol_ref = Some(item_ref);
    symbol.bankdef_ref = Some(ctx.bank_ref);

    asm::resolver::handle_value_resolution(
        opts,
        report,
        ast_symbol.decl_span,
        ctx.can_guess(),
        symbol.value.is_guess(),
        is_stable,
        &mut symbol.resolved,
        false,
        "label",
        "label address",
        Some(&ast_symbol.name),
        &symbol.value)
}