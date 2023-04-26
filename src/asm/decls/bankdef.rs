use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    decls: &mut asm::ItemDecls)
    -> Result<(), ()>
{
    let initial_item_ref = decls.bankdefs.declare(
        report,
        diagn::Span::new_dummy(),
        &util::SymbolContext::new_global(),
        "#global_bankdef".to_string(),
        0,
        util::SymbolKind::Other)?;
    
    debug_assert!(initial_item_ref.0 == 0);


    for any_node in &mut ast.nodes
    {
        if let asm::AstAny::DirectiveBankdef(ref mut node) = any_node
        {
            let item_ref = decls.bankdefs.declare(
                report,
                node.name_span,
                &util::SymbolContext::new_global(),
                node.name.clone(),
                0,
                util::SymbolKind::Other)?;
                
            node.item_ref = Some(item_ref);
        }
    }


    Ok(())
}