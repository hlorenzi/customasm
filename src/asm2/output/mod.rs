use crate::*;


pub fn check_bank_overlap(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs)
    -> Result<(), ()>
{
    for i in 1..defs.bankdefs.len()
    {
        let bankdef1 = &defs.bankdefs.defs[i];
        let decl1 = decls.bankdefs.get(bankdef1.item_ref);

        if bankdef1.output_offset.is_none()
        {
            continue;
        }

        for j in (i + 1)..defs.bankdefs.len()
        {
            let bankdef2 = &defs.bankdefs.defs[j];
            let decl2 = decls.bankdefs.get(bankdef2.item_ref);

            if bankdef2.output_offset.is_none()
            {
                continue;
            }

            let outp1 = bankdef1.output_offset.unwrap();
            let outp2 = bankdef2.output_offset.unwrap();

            // FIXME: multiplication can overflow
            let size1 = bankdef1.size;

            let size2 = bankdef2.size;

            let overlap = {
                match (size1, size2)
                {
                    (None, None) =>
                        true,

                    (Some(size1), None) =>
                        outp1 + size1 > outp2,

                    (None, Some(size2)) =>
                        outp2 + size2 > outp1,

                    (Some(size1), Some(size2)) =>
                        outp1 + size1 > outp2 && outp2 + size2 > outp1,
                }
            };

            if overlap
            {
                report.push_parent(
                    format!(
                        "output of bank `{}` overlaps with bank `{}`",
                        decl1.name,
                        decl2.name),
                    &decl1.span);

                report.note_span(
                    format!(
                        "bank `{}` defined here",
                        decl2.name),
                    &decl2.span);
                    
                return Err(());
            }
        }
    }

    Ok(())
}


pub fn build_output(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs)
    -> Result<util::BitVec, ()>
{
    let mut bitvec = util::BitVec::new();

    fill_banks(
        defs,
        &mut bitvec);

    let mut iter = asm2::ResolveIterator::new(
        ast,
        defs,
        false,
        true);

    while let Some(ctx) = iter.next(decls, defs)
    {
        if let asm2::ResolverNode::Symbol(ast_symbol) = ctx.node
        {
            let symbol = defs.symbols.get(ast_symbol.item_ref.unwrap());

            if let asm2::AstSymbolKind::Label = ast_symbol.kind
            {
                check_bank_usage(
                    report,
                    &ast_symbol.decl_span,
                    defs,
                    &ctx)?;

                check_bank_output(
                    report,
                    &ast_symbol.decl_span,
                    decls,
                    defs,
                    &ctx,
                    0,
                    false)?;

                let maybe_pos = ctx.get_output_position(defs);

                bitvec.mark_span(
                    maybe_pos,
                    0,
                    symbol.value.unwrap_bigint().clone(),
                    ast_symbol.decl_span.clone());
            }
        }
        
        else if let asm2::ResolverNode::Instruction(ast_instr) = ctx.node
        {
            let instr = defs.instructions.get(ast_instr.item_ref.unwrap());

            check_bank_usage(
                report,
                &ast_instr.span,
                defs,
                &ctx)?;

            check_bank_output(
                report,
                &ast_instr.span,
                decls,
                defs,
                &ctx,
                instr.encoding.size.unwrap(),
                true)?;
                
            let addr = ctx.get_address(defs, true).unwrap();
            let pos = ctx.get_output_position(defs).unwrap();

			bitvec.write_bigint_checked(
                report,
				&ast_instr.span,
                pos,
				addr,
                &instr.encoding)?;
        }
        
        else if let asm2::ResolverNode::DataElement(ast_data, elem_index) = ctx.node
        {
            let item_ref = ast_data.item_refs[elem_index];
            let elem = defs.data_elems.get(item_ref);

            check_bank_usage(
                report,
                ast_data.elems[elem_index].span(),
                defs,
                &ctx)?;

            check_bank_output(
                report,
                ast_data.elems[elem_index].span(),
                decls,
                defs,
                &ctx,
                elem.encoding.size.unwrap(),
                true)?;
                
            let pos = ctx.get_output_position(defs).unwrap();
            let addr = ctx.get_address(defs, true).unwrap();

            bitvec.write_bigint_checked(
                report,
                ast_data.elems[elem_index].span(),
                pos,
                addr,
                &elem.encoding)?;
        }
        
        else if let asm2::ResolverNode::Res(ast_res) = ctx.node
        {
            let item_ref = ast_res.item_ref.unwrap();
            let res = defs.res_directives.get(item_ref);

            check_bank_usage(
                report,
                &ast_res.header_span,
                defs,
                &ctx)?;

            check_bank_output(
                report,
                &ast_res.header_span,
                decls,
                defs,
                &ctx,
                res.reserve_size,
                false)?;
        }
    }

    Ok(bitvec)
}


fn fill_banks(
    defs: &asm2::ItemDefs,
    bitvec: &mut util::BitVec)
{
    for bankdef in &defs.bankdefs.defs
    {
        if !bankdef.fill
        {
            continue;
        }

        if let (Some(size), Some(output)) = (bankdef.size, bankdef.output_offset)
        {
            let highest_position = output + size - 1;

            if bitvec.len() < highest_position
            {
                bitvec.write(highest_position, false);
            }
        }
    }
}


fn check_bank_usage(
    report: &mut diagn::Report,
    span: &diagn::Span,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<(), ()>
{
    if ctx.bank_ref.0 == 0
    {
        if defs.bankdefs.defs.len() == 1
        {
            return Ok(());
        }

        report.error_span(
            "using the default bank while custom banks are defined",
            span);

        return Err(());
    }

    Ok(())
}


fn check_bank_output(
    report: &mut diagn::Report,
    span: &diagn::Span,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    size: usize,
    write: bool)
    -> Result<(), ()>
{
    let bankdef = defs.bankdefs.get(ctx.bank_ref);
    let bankdef_decl = decls.bankdefs.get(ctx.bank_ref);

    if let Some(bank_size) = bankdef.size
    {
        // FIXME: Addition can overflow
        if ctx.bank_data.cur_position + size > bank_size
        {
            report.push_parent(
                format!(
                    "output out of range for bank `{}`",
                    bankdef_decl.name),
                span);

            report.note_span(
                "bank defined here:",
                &bankdef_decl.span);
    
            report.pop_parent();
    
            return Err(());
        }
    }

    if write && bankdef.output_offset.is_none()
    {
        report.push_parent(
            format!(
                "output to non-writable bank `{}`",
                bankdef_decl.name),
            span);

        report.note_span(
            "no `outp` defined for bank",
            &bankdef_decl.span);

        report.pop_parent();

        return Err(());
    }

    Ok(())
}