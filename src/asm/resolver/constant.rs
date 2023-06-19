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
    if symbol.resolved
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
        symbol.resolved = true;
        return Ok(asm::ResolutionState::Resolved);
    }


    let value = asm::resolver::eval_simple(
        report,
        decls,
        defs,
        &ast_const.expr)?;


    let symbol = defs.symbols.get_mut(item_ref);
    symbol.value = value;


    if symbol.value.is_unknown()
    {
        return Ok(asm::ResolutionState::Unresolved);
    }


    // Optimize future iterations for the case where it's
    // statically known that the encoding can be resolved
    // in the first pass
    if opts.optimize_statically_known &&
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


    if opts.debug_iterations
    {
        println!("const: {} = {:?}",
            ast_symbol.name,
            symbol.value);
    }
    
    Ok(asm::ResolutionState::Resolved)
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
    if symbol.resolved
    {
        return Ok(asm::ResolutionState::Resolved);
    }

    let asm::AstSymbolKind::Constant(ref ast_const) = ast_symbol.kind
        else { unreachable!() };
        
    let value = asm::resolver::eval(
        report,
        opts,
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
                ast_symbol.decl_span);
        }

        if opts.debug_iterations
        {
            println!("const: {} = {:?}",
                ast_symbol.name,
                symbol.value);
        }

        return Ok(asm::ResolutionState::Unresolved);
    }

    
    Ok(asm::ResolutionState::Resolved)
}