use crate::*;


pub fn resolve_data_block(
    report: &mut diagn::Report,
    ast_data: &asm2::AstDirectiveData,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let mut resolution_state = asm2::ResolutionState::Resolved;

    for i in 0..ast_data.item_refs.len()
    {
        resolution_state.merge(resolve_data_block_element(
            report,
            ast_data,
            i,
            decls,
            defs,
            ctx)?);
    }

    Ok(resolution_state)
}


pub fn resolve_data_block_element(
    report: &mut diagn::Report,
    ast_data: &asm2::AstDirectiveData,
    elem_index: usize,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let expr = &ast_data.elems[elem_index];

    report.push_parent(
        "failed to resolve data element",
        expr.span());
    
    let maybe_encoding = asm2::resolver::eval(
        report,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext2::new(),
        expr);

    report.pop_parent();

            
    let bigint = {
        match maybe_encoding?.expect_error_or_bigint(
            report,
            expr.span())?
        {
            expr::Value::Integer(i) => i,

            expr::Value::Unknown =>
            {
                if ctx.is_last_iteration
                {
                    report.error_span(
                        "failed to resolve data element",
                        expr.span());
                    
                    return Ok(asm2::ResolutionState::Unresolved);
                }

                util::BigInt::new(0, ast_data.elem_size)
            }

            expr::Value::FailedConstraint(msg) =>
            {
                if ctx.is_last_iteration
                {
                    report.message(msg.clone());
                    return Ok(asm2::ResolutionState::Unresolved);
                }

                util::BigInt::new(0, ast_data.elem_size)
            }

            _ => unreachable!(),
        }
    };


    if let Some(size) = ast_data.elem_size
    {
        let bigint_size = bigint.size_or_min_size();

        if ctx.is_last_iteration &&
            bigint_size > size
        {
            report.error_span(
                format!(
                    "value size (= {}) out of range for directive size (= {})",
                    bigint_size,
                    size),
                expr.span());

            return Ok(asm2::ResolutionState::Unresolved);
        }
    }


    let bigint = {
        if let Some(size) = ast_data.elem_size
        {
            bigint.slice(size, 0)
        }
        else
        {
            bigint.slice(bigint.size_or_min_size(), 0)
        }
    };


    let item_ref = ast_data.item_refs[elem_index];
    let data_elem = defs.data_elems.get_mut(item_ref);
    let prev_encoding = data_elem.encoding.clone();
    data_elem.encoding = bigint;


    if data_elem.encoding != prev_encoding
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span(
                "data element did not converge",
                expr.span());
        }
        
        println!(" data: {:?}", data_elem.encoding);
        return Ok(asm2::ResolutionState::Unresolved);
    }


    Ok(asm2::ResolutionState::Resolved)
}