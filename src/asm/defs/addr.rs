use crate::*;

#[derive(Debug)]
pub struct AddrDirective
{
    pub item_ref: util::ItemRef<Self>,
    pub address: util::BigInt,
}

pub fn define(
    _report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    _decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs,
) -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm::AstAny::DirectiveAddr(ref mut ast_addr) = any_node
        {
            let item_ref = defs.addr_directives.next_item_ref();

            let res = AddrDirective {
                item_ref,
                address: util::BigInt::new(0, None),
            };

            defs.addr_directives.define(item_ref, res);

            ast_addr.item_ref = Some(item_ref);
        }
    }

    Ok(())
}
