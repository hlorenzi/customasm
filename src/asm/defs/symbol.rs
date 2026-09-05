use crate::*;


#[derive(Debug)]
pub struct Symbol
{
    pub item_ref: util::ItemRef<Self>,
    pub no_emit: bool,
    pub value: expr::Value,
    pub resolved: bool,
    pub driver_defined: bool,
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
        };

        defs.symbols.define(item_ref, symbol);
    }


    Ok(())
}


pub fn update_bank_refs(
    _report: &mut diagn::Report,
    _opts: &asm::AssemblyOptions,
    ast: &asm::AstTopLevel,
    decls: &mut asm::ItemDecls,
    _defs: &mut asm::ItemDefs)
{
    let mut bank_ref = None;

    for any_node in &ast.nodes
    {
        match any_node
        {
            asm::AstAny::DirectiveBank(ast_bank) =>
            {
                bank_ref = ast_bank.item_ref;
            }

            asm::AstAny::DirectiveBankdef(ast_bankdef) =>
            {
                bank_ref = Some(ast_bankdef.item_ref.unwrap());
            }

            asm::AstAny::Symbol(ast_symbol) =>
            {
                let decl = decls.symbols
                    .get_mut(ast_symbol.item_ref.unwrap());

                decl.bank_ref = bank_ref;
            }

            _ => {}
        }
    }
}