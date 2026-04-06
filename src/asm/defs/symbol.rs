use crate::*;


#[derive(Debug)]
pub struct Symbol
{
    pub item_ref: util::ItemRef<Self>,
    pub no_emit: bool,
    pub value: expr::Value,
    pub resolved: bool,
    pub driver_defined: bool,
    pub bankdef_ref: Option<util::ItemRef<asm::Bankdef>>,
}


pub fn define(
    _report: &mut diagn::Report,
    _opts: &asm::AssemblyOptions,
    ast: &asm::AstTopLevel,
    _decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        let asm::AstAny::Symbol(node) = any_node
            else { continue };

        if defs.symbols
            .maybe_get(node.item_ref.unwrap())
            .is_some()
        {
            continue;
        }


        let item_ref = node.item_ref.unwrap();

        let symbol = Symbol {
            item_ref,
            no_emit: node.no_emit,
            value: expr::Value::make_unknown()
                .with_symbol_ref(item_ref),
            resolved: false,
            driver_defined: false,
            bankdef_ref: None,
        };

        defs.symbols.define(item_ref, symbol);
    }


    Ok(())
}