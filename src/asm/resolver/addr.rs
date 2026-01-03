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

    let value = asm::resolver::eval(
        report,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(opts),
        &ast_addr.expr)?;

    let value = value.expect_error_or_bigint(
        report,
        ast_addr.expr.span())?;

    let value = {
        match value
        {
            expr::Value::Integer(_, bigint) => bigint,
            _ => util::BigInt::new(0, None),
        }
    };


    let addr = defs.addr_directives.get_mut(item_ref);
    let prev_value = addr.address.clone();
    addr.address = value;


    if addr.address != prev_value
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "address did not converge",
                ast_addr.expr.span());
        }

        if opts.debug_iterations
        {
            println!(" addr: {:?}", addr.address);
        }
        
        return Ok(asm::ResolutionState::Unresolved);
    }


    if ctx.is_last_iteration
    {
        let bank = defs.bankdefs.get(ctx.bank_ref);

        if addr.address < bank.addr_start
        {
            report.error_span(
                "address is out of bank range",
                ast_addr.expr.span());

            return Err(());
        }

        let addr_size = &addr.address.checked_sub(
            report,
            ast_addr.header_span,
            &bank.addr_start)?;

        let addr_delta = addr_size
            .checked_mul(
                report,
                ast_addr.header_span,
                &util::BigInt::new(bank.addr_unit, None))?
            .checked_into::<usize>(
                report,
                ast_addr.header_span)?;


        if let Some(size) = bank.size_in_bits
        {
            if addr_delta >= size
            {
                report.error_span(
                    "address is out of bank range",
                    ast_addr.expr.span());

                return Err(());
            }
        }
    }

    
    Ok(asm::ResolutionState::Resolved)
}