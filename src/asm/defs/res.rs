use crate::*;

#[derive(Debug)]
pub struct ResDirective
{
    pub item_ref: util::ItemRef<Self>,
    pub reserve_size: usize,
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
        if let asm::AstAny::DirectiveRes(ref mut ast_res) = any_node
        {
            let item_ref = defs.res_directives.next_item_ref();

            let res = ResDirective {
                item_ref,
                reserve_size: 0,
            };

            defs.res_directives.define(item_ref, res);

            ast_res.item_ref = Some(item_ref);
        }
    }

    Ok(())
}
