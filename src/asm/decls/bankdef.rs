use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    decls: &mut asm::ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        let asm::AstAny::DirectiveBankdef(ref mut node) = any_node
            else { continue };

        if node.item_ref.is_some()
        {
            continue;
        }
        

        let item_ref = decls.bankdefs.declare(
            report,
            node.name_span,
            &util::SymbolContext::new_global(),
            node.name.clone(),
            0,
            util::SymbolKind::Other)?;
            
        node.item_ref = Some(item_ref);
    }


    Ok(())
}