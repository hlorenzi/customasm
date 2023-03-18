use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::DirectiveBank(ref mut node) = any_node
        {
            let item_ref = decls.banks.get_by_name_global(
                report,
                &node.name_span,
                &node.name)?;
                
            node.item_ref = Some(item_ref);
        }
    }


    Ok(())
}