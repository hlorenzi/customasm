use crate::*;


pub fn resolve_addr(
    report: &mut diagn::Report,
    opts: &asm2::AssemblyOptions,
    fileserver: &dyn util::FileServer,
    ast_addr: &asm2::AstDirectiveAddr,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let item_ref = ast_addr.item_ref.unwrap();

    let value = asm2::resolver::eval(
        report,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext2::new(),
        &ast_addr.expr)?;

    let value = value.expect_error_or_bigint(
        report,
        &ast_addr.expr.span())?;

    let value = {
        match value
        {
            expr::Value::Integer(bigint) => bigint,
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
                &ast_addr.expr.span());
        }

        if opts.debug_iterations
        {
            println!(" addr: {:?}", addr.address);
        }
        
        return Ok(asm2::ResolutionState::Unresolved);
    }


    if ctx.is_last_iteration
    {
        let bank = defs.bankdefs.get(ctx.bank_ref);

        if addr.address < bank.addr_start
        {
            report.error_span(
                "address is out of bank range",
                &ast_addr.expr.span());

            return Err(());
        }

        let addr_size = &addr.address - &bank.addr_start;

        let maybe_addr_delta =
            (&addr_size * &util::BigInt::new(bank.addr_unit, None))
            .checked_to_usize();

        if maybe_addr_delta.is_none()
        {
            report.error_span(
                "value is out of supported range",
                &ast_addr.expr.span());

            return Err(());
        }

        else if let Some(size) = bank.size
        {
            if maybe_addr_delta.is_none() ||
                maybe_addr_delta.unwrap() >= size
            {
                report.error_span(
                    "address is out of bank range",
                    &ast_addr.expr.span());

                return Err(());
            }
        }
    }

    
    Ok(asm2::ResolutionState::Resolved)
}