use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls)
    -> Result<(), ()>
{
    let mut symbol_ctx = util::SymbolContext::new_global();


    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::Constant(ref mut node) = any_node
        {
            let (item_ref, new_ctx) = decls.symbols.declare(
                report,
                &node.decl_span,
                &symbol_ctx,
                node.name.clone(),
                node.hierarchy_level)?;
                
            node.item_ref = Some(item_ref);

            symbol_ctx = new_ctx;
        }
        
        else if let asm2::AstAny::Label(ref mut node) = any_node
        {
            let (item_ref, new_ctx) = decls.symbols.declare(
                report,
                &node.decl_span,
                &symbol_ctx,
                node.name.clone(),
                node.hierarchy_level)?;
                
            node.item_ref = Some(item_ref);

            symbol_ctx = new_ctx;
        }
    }


    Ok(())
}