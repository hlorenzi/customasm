use crate::*;


pub fn resolve_constants_simple(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    _fileserver: &mut dyn util::FileServer,
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

    while let Some(ctx) = iter.next_simple(report, decls, defs)?
    {
        let asm::ResolverNode::Symbol(ast_symbol) = ctx.node
            else { continue };
        
        let asm::AstSymbolKind::Constant(_) = ast_symbol.kind
            else { continue };

        let resolution_state = resolve_constant_simple(
            report,
            opts,
            ast_symbol,
            decls,
            defs)?;

        if let asm::ResolutionState::Resolved = resolution_state
        {
            resolved_count += 1;
        }
    }

    Ok(resolved_count)
}


fn resolve_constant_simple(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast_symbol: &asm::AstSymbol,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_symbol.item_ref.unwrap();

    let symbol = defs.symbols.get(item_ref);

    if (symbol.resolved && opts.optimize_statically_known) ||
        symbol.driver_defined
    {
        return Ok(asm::ResolutionState::Resolved);
    }

    let asm::AstSymbolKind::Constant(ref ast_const) = ast_symbol.kind
        else { unreachable!() };

    // Overwrite with a value from the command-line, if present
    let symbol_decl = decls.symbols.get(item_ref);
    if let Some(driver_def) = opts.driver_symbol_defs
        .iter()
        .find(|s| s.name == symbol_decl.name)
    {
        let symbol = defs.symbols.get_mut(item_ref);
        symbol.value = driver_def.value.clone();
        symbol.value.get_mut_metadata().symbol_ref = Some(item_ref);
        symbol.driver_defined = true;
        symbol.resolved = true;
        
        return asm::resolver::handle_value_resolution(
            opts,
            report,
            ast_symbol.decl_span,
            false,
            false,
            true,
            &mut symbol.resolved,
            false,
            "driver const",
            "constant value",
            Some(&ast_symbol.name),
            &symbol.value);
    }

    let value = asm::resolver::eval_simple(
        report,
        opts,
        decls,
        defs,
        &ast_const.expr)?;

    let symbol = defs.symbols.get_mut(item_ref);
    let is_stable = value.is_stable(&symbol.value);
    symbol.value = value;
    symbol.value.get_mut_metadata().symbol_ref = Some(item_ref);

    asm::resolver::handle_value_resolution(
        opts,
        report,
        ast_symbol.decl_span,
        true,
        symbol.value.is_guess(),
        is_stable,
        &mut symbol.resolved,
        false,
        "const",
        "constant value",
        Some(&ast_symbol.name),
        &symbol.value)
}


pub fn resolve_constant(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_symbol: &asm::AstSymbol,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_symbol.item_ref.unwrap();

    let symbol = defs.symbols.get(item_ref);
    
    if (symbol.resolved && opts.optimize_statically_known) ||
        symbol.driver_defined
    {
        return Ok(asm::ResolutionState::Resolved);
    }

    let asm::AstSymbolKind::Constant(ref ast_const) = ast_symbol.kind
        else { unreachable!() };
    
    let value = asm::resolver::eval(
        report,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(opts),
        &ast_const.expr)?;

    let symbol = defs.symbols.get_mut(item_ref);
    let is_stable = value.is_stable(&symbol.value);
    symbol.value = value;
    symbol.value.get_mut_metadata().symbol_ref = Some(item_ref);

    asm::resolver::handle_value_resolution(
        opts,
        report,
        ast_symbol.decl_span,
        ctx.can_guess(),
        symbol.value.is_guess(),
        is_stable,
        &mut symbol.resolved,
        false,
        "const",
        "constant value",
        Some(&ast_symbol.name),
        &symbol.value)
}