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
            let size1 = bankdef1.addr_size
                .map(|s| s * bankdef1.addr_unit);

            let size2 = bankdef2.addr_size
                .map(|s| s * bankdef2.addr_unit);

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
            let maybe_pos = ctx.get_output_position(defs);

            if let asm2::AstSymbolKind::Label = ast_symbol.kind
            {
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
            let pos = ctx.get_output_position(defs).unwrap();
            let addr = ctx.get_address(defs, true).unwrap();

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
            let pos = ctx.get_output_position(defs).unwrap();
            let addr = ctx.get_address(defs, true).unwrap();

            bitvec.write_bigint_checked(
                report,
                ast_data.elems[elem_index].span(),
                pos,
                addr,
                &elem.encoding)?;
        }
    }

    Ok(bitvec)
}