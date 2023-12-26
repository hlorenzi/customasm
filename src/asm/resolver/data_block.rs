use crate::*;

pub fn resolve_data_element(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_data: &asm::AstDirectiveData,
    elem_index: usize,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext,
) -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_data.item_refs[elem_index];
    let data_elem = defs.data_elems.get(item_ref);

    if data_elem.resolved
    {
        return Ok(asm::ResolutionState::Resolved);
    }

    let expr = &ast_data.elems[elem_index];

    report.push_parent("failed to resolve data element", expr.span());

    let maybe_value = asm::resolver::eval(
        report,
        opts,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(),
        expr,
    );

    report.pop_parent();

    let maybe_encoding = {
        match maybe_value?.expect_error_or_bigint(report, expr.span())?
        {
            expr::Value::Integer(i) => Some(i),

            expr::Value::Unknown =>
            {
                if ctx.is_last_iteration || data_elem.encoding_statically_known
                {
                    report.error_span("failed to resolve data element", expr.span());

                    return Err(());
                }

                None
            }

            expr::Value::FailedConstraint(msg) =>
            {
                if ctx.is_last_iteration || data_elem.encoding_statically_known
                {
                    report.message(msg.clone());
                    return Err(());
                }

                None
            }

            _ => unreachable!(),
        }
    };

    if ctx.is_last_iteration || data_elem.encoding_statically_known
    {
        let encoding = maybe_encoding.as_ref().unwrap();

        // Check the element size against the directive size
        if let Some(elem_size) = ast_data.elem_size
        {
            let encoding_size = encoding.size_or_min_size();

            if encoding_size > elem_size
            {
                report.push_parent("value out of range for directive", expr.span());

                report.note(format!(
                    "data directive has size {}, got size {}",
                    elem_size, encoding_size
                ));

                report.pop_parent();

                return Err(());
            }
        }

        // Check for definite size
        if ast_data.elem_size.is_none() && encoding.size.is_none()
        {
            report.error_span("data element has no definite size", expr.span());

            return Err(());
        }
    }

    // Apply definite size via slice
    let maybe_encoding = {
        maybe_encoding.map(|e| {
            if let Some(elem_size) = ast_data.elem_size
            {
                e.slice(elem_size, 0)
            }
            else
            {
                e.slice(e.size_or_min_size(), 0)
            }
        })
    };

    let data_elem = defs.data_elems.get_mut(item_ref);
    let prev_encoding = data_elem.encoding.clone();

    if let Some(ref encoding) = maybe_encoding
    {
        data_elem.encoding = encoding.clone();

        // Optimize future iterations for the case where it's
        // statically known that the encoding can be resolved
        // in the first pass
        if opts.optimize_statically_known
            && ctx.is_first_iteration
            && data_elem.encoding_statically_known
            && encoding.size.is_some()
        {
            if opts.debug_iterations
            {
                println!(
                    " data: {} = {:?} [static]",
                    fileserver.get_excerpt(expr.span()),
                    data_elem.encoding
                );
            }

            data_elem.resolved = true;
            return Ok(asm::ResolutionState::Resolved);
        }
    }

    if Some(&prev_encoding) != maybe_encoding.as_ref()
    {
        // On the final iteration, unstable guesses become errors
        if ctx.is_last_iteration
        {
            report.error_span("data element did not converge", expr.span());
        }

        if opts.debug_iterations
        {
            println!(
                " data: {} = {:?}",
                fileserver.get_excerpt(expr.span()),
                data_elem.encoding
            );
        }

        return Ok(asm::ResolutionState::Unresolved);
    }

    Ok(asm::ResolutionState::Resolved)
}
