use crate::*;


pub fn resolve_addr(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_addr: &asm::AstDirectiveAddr,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_addr.item_ref.unwrap();

    let addr = defs.addr_directives.get(item_ref);

    if addr.resolved && opts.optimize_statically_known {
        return Ok(asm::ResolutionState::Resolved);
    }
    
    let value = asm::resolver::eval(
        report,
        fileserver,
        opts,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(opts),
        &ast_addr.expr)?;

    let addr = defs.addr_directives.get_mut(item_ref);
    let is_stable = value.is_stable(&addr.value);
    addr.value = value;

    let offset_from_bank_start_in_bits = finalize_addr(
        report,
        ast_addr,
        defs,
        ctx)?;

    let addr = defs.addr_directives.get_mut(item_ref);
    addr.offset_from_bank_start_in_bits = offset_from_bank_start_in_bits;

    asm::resolver::handle_value_resolution(
        opts,
        report,
        ast_addr.expr.span(),
        ctx.can_guess(),
        addr.value.is_guess(),
        is_stable,
        &mut addr.resolved,
        false,
        "addr",
        "address",
        None,
        &addr.value)
}


pub fn finalize_addr(
    report: &mut diagn::Report,
    ast_addr: &asm::AstDirectiveAddr,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<usize, ()>
{
    let item_ref = ast_addr.item_ref.unwrap();

    let addr = defs.addr_directives.get(item_ref);

    if let expr::Value::Integer(_, bigint) = &addr.value
    {
        let bank = defs.bankdefs.get(ctx.bank_ref);

        if bigint < &bank.addr_start
        {
            if ctx.can_guess() {
                return Ok(0);
            }
            
            report.error_span(
                "address is out of bank range",
                ast_addr.expr.span());

            return Err(());
        }

        let offset_from_bank_start = bigint
            .maybe_sub(&bank.addr_start)
            .and_then(|a| a.maybe_mul(&util::BigInt::new(bank.addr_unit, None)))
            .and_then(|a| a.maybe_into::<usize>());
        
        if let Some(offset_from_bank_start) = offset_from_bank_start
        {
            if let Some(bank_size_in_bits) = bank.size_in_bits
            {
                if offset_from_bank_start >= bank_size_in_bits
                {
                    report.error_span(
                        "address is out of bank range",
                        ast_addr.expr.span());

                    return Err(());
                }
            }

            return Ok(offset_from_bank_start);
        }
        else
        {
            if ctx.can_guess() {
                return Ok(0);
            }
            
            report.error_span(
                "address is outside the supported range",
                ast_addr.expr.span());
        }
    }
    else
    {
        if ctx.can_guess() {
            return Ok(0);
        }
        
        report.error_span(
            format!(
                "invalid type for address (have {})",
                addr.value.type_name()),
            ast_addr.expr.span());
    }

    Err(())
}