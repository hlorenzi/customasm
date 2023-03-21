use crate::*;


#[derive(Debug)]
pub struct Symbol
{
    pub item_ref: util::ItemRef<Self>,
    pub value: expr::Value,
    pub value_guess: expr::Value,
}


pub fn define(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        if let asm2::AstAny::Symbol(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();

            let symbol = Symbol {
                item_ref,
                value: expr::Value::Unknown,
                value_guess: expr::Value::make_integer(0),
            };

            defs.symbols.define(item_ref, symbol);
        }
    }


    Ok(())
}