use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::DirectiveRuledef(ref mut node) = any_node
        {
            let name = node.name
                .clone()
                .unwrap_or_else(||
                    decls.ruledefs.generate_anonymous_name());


            let item_ref = decls.ruledefs.declare(
                report,
                &node.name_span,
                &util::SymbolContext::new_global(),
                name,
                0)?;
                
            node.item_ref = Some(item_ref);
        }
    }


    Ok(())
}