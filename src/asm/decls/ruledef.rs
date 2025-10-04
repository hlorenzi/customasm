use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    decls: &mut asm::ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        let asm::AstAny::DirectiveRuledef(node) = any_node
            else { continue };

        if node.item_ref.is_some()
        {
            continue;
        }


        let name = node.name
            .clone()
            .unwrap_or_else(||
                decls.ruledefs.generate_anonymous_name());

        let item_ref = decls.ruledefs.declare(
            report,
            node.name_span,
            &util::SymbolContext::new_global(),
            None,
            name,
            0,
            util::SymbolKind::Other)?;
            
        node.item_ref = Some(item_ref);
    }


    Ok(())
}