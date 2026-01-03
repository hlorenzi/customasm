use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    decls: &mut asm::ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        let asm::AstAny::DirectiveBank(node) = any_node
            else { continue };

        if node.item_ref.is_some()
        {
            continue;
        }

        let item_ref = decls.bankdefs.get_by_name_global(
            report,
            node.name_span,
            &node.name)?;
            
        node.item_ref = Some(item_ref);
    }


    Ok(())
}