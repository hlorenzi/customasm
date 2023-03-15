use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::DirectiveBankdef(ref mut node) = any_node
        {
            let (item_ref, _) = decls.banks.declare(
                report,
                &node.name_span,
                &util::SymbolContext::new_global(),
                node.name.clone(),
                0)?;
                
            node.item_ref = Some(item_ref);
        }
    }


    Ok(())
}