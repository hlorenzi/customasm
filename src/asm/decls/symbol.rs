use crate::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    decls: &mut asm::ItemDecls)
    -> Result<(), ()>
{
    let mut symbol_ctx = util::SymbolContext::new_global();

    let mut bank_ref = None;


    for any_node in &mut ast.nodes
    {
        match any_node
        {
            asm::AstAny::DirectiveBank(ast_bank) =>
            {
                bank_ref = Some(ast_bank.item_ref.unwrap());
            }

            asm::AstAny::DirectiveBankdef(ast_bankdef) =>
            {
                bank_ref = Some(ast_bankdef.item_ref.unwrap());
            }

            asm::AstAny::Symbol(node) =>
            {
                if node.item_ref.is_none()
                {
                    let kind = {
                        match node.kind
                        {
                            asm::AstSymbolKind::Label =>
                                util::SymbolKind::Label,
                            asm::AstSymbolKind::Constant(_) =>
                                util::SymbolKind::Constant,
                        }
                    };

                    let item_ref = decls.symbols.declare(
                        report,
                        node.decl_span,
                        &symbol_ctx,
                        bank_ref,
                        node.name.clone(),
                        node.hierarchy_level,
                        kind)?;
                        
                    node.item_ref = Some(item_ref);
                }

                symbol_ctx = decls.symbols
                    .get(node.item_ref.unwrap())
                    .ctx
                    .clone();
            }

            _ => {}
        }
    }


    Ok(())
}