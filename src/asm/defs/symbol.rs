use crate::*;


#[derive(Debug)]
pub struct Symbol
{
    pub item_ref: util::ItemRef<Self>,
    pub value: expr::Value,
    pub bankdef_ref: Option<util::ItemRef<asm::Bankdef>>,
}


pub fn define(
    _report: &mut diagn::Report,
    ast: &asm::AstTopLevel,
    _decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        if let asm::AstAny::Symbol(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();

            let symbol = Symbol {
                item_ref,
                value: expr::Value::Unknown,
                bankdef_ref: None,
            };

            defs.symbols.define(item_ref, symbol);
        }
    }


    Ok(())
}