use crate::*;


#[derive(Debug)]
pub struct ResDirective
{
    pub item_ref: util::ItemRef<Self>,
    pub reserve_size: usize,
}


pub fn define(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::DirectiveRes(ref mut ast_res) = any_node
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