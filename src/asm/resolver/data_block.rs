use crate::*;


pub fn resolve_data_element(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_data: &asm::AstDirectiveData,
    elem_index: usize,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let item_ref = ast_data.item_refs[elem_index];
    let data_elem = defs.data_elems.get(item_ref);

    if data_elem.resolved && opts.optimize_statically_known {
        return Ok(asm::ResolutionState::Resolved);
    }

    let expr = &ast_data.elems[elem_index];

    report.push_parent(
        "failed to resolve data element",
        expr.span());
    
    let maybe_value = asm::resolver::eval(
        report,
        fileserver,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new(opts),
        expr);

    report.pop_parent();

    let value = maybe_value?;

    let data_elem = defs.data_elems.get_mut(item_ref);
    let is_stable = value.is_stable(&data_elem.encoding);
    data_elem.encoding = value;

    asm::resolver::handle_value_resolution(
        opts,
        report,
        ast_data.elems[elem_index].span(),
        ctx.can_guess(),
        data_elem.encoding.is_guess(),
        is_stable,
        &mut data_elem.resolved,
        false,
        "data",
        "data element value",
        None,
        &data_elem.encoding)
}


pub fn check_final_data_element(
    report: &mut diagn::Report,
    span: diagn::Span,
    elem: &asm::DataElement)
    -> Result<util::BigInt, ()>
{
    if let expr::Value::Integer(_, ref encoding) = elem.encoding
    {
        // Check the element size against the directive size
        if let Some(elem_size) = elem.elem_size
        {
            let encoding_size = encoding.size_or_min_size();
            
            if encoding_size > elem_size
            {
                report.push_parent(
                    "value out of range for directive",
                    span);

                report.note(
                    format!(
                        "data directive has size {}, got size {}",
                        elem_size,
                        encoding_size));

                report.pop_parent();

                return Err(());
            }
        }
        
        // Check for definite size
        if elem.elem_size.is_none() &&
            encoding.size.is_none()
        {
            report.error_span(
                "data element has no definite size",
                span);

            return Err(());
        }

        // Apply definite size via slice
        let encoding = {
            if let Some(elem_size) = elem.elem_size {
                encoding.slice(elem_size, 0)
            }
            else {
                encoding.slice(encoding.size_or_min_size(), 0)
            }
        };

        Ok(encoding)
    }
    else
    {
        report.error_span(
            format!(
                "invalid type for data element (have {})",
                elem.encoding.type_name()),
            span);

        Err(())
    }
}