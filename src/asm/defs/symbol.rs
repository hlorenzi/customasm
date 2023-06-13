use crate::*;


#[derive(Debug)]
pub struct Symbol
{
    pub item_ref: util::ItemRef<Self>,
    pub no_emit: bool,
    pub value_statically_known: bool,
    pub value: expr::Value,
    pub resolved: bool,
    pub bankdef_ref: Option<util::ItemRef<asm::Bankdef>>,
}


pub fn define(
    _report: &mut diagn::Report,
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

        let value_statically_known = {
            match node.kind
            {
                asm::AstSymbolKind::Constant(ref constant) =>
                {
                    let mut provider = expr::StaticallyKnownProvider::new();
                    provider.query_function = &asm::resolver::get_statically_known_builtin_fn;
                    
                    constant.expr.is_value_statically_known(&provider)
                }
                
                _ => false,
            }
        };

        let symbol = Symbol {
            item_ref,
            no_emit: node.no_emit,
            value_statically_known,
            value: expr::Value::Unknown,
            resolved: false,
            bankdef_ref: None,
        };

        defs.symbols.define(item_ref, symbol);
    }


    Ok(())
}