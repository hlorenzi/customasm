use crate::*;


#[derive(Debug)]
pub struct AddrDirective
{
    pub item_ref: util::ItemRef<Self>,
    pub value: expr::Value,
    pub offset_from_bank_start_in_bits: usize,
    pub resolved: bool,
}


pub fn define(
    _report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    _decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm::AstAny::DirectiveAddr(ast_addr) = any_node
        {
            let item_ref = defs.addr_directives.next_item_ref();

            let res = AddrDirective {
                item_ref,
                value: expr::Value::make_unknown(),
                offset_from_bank_start_in_bits: 0,
                resolved: false,
            };
            
            defs.addr_directives.define(item_ref, res);
                
            ast_addr.item_ref = Some(item_ref);
        }
    }


    Ok(())
}