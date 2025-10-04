use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    decls: &mut asm::ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        let asm::AstAny::DirectiveFn(node) = any_node
            else { continue };

        if node.item_ref.is_some()
        {
            continue;
        }
        

        let item_ref = decls.symbols.declare(
            report,
            node.name_span,
            &util::SymbolContext::new_global(),
            None,
            node.name.clone(),
            0,
            util::SymbolKind::Function)?;
            
        node.item_ref = Some(item_ref);
    }


    Ok(())
}