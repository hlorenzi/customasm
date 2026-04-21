use crate::*;


#[derive(Debug)]
pub struct AlignDirective
{
    pub item_ref: util::ItemRef<Self>,
    pub align_size: usize,
    pub value: expr::Value,
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
        if let asm::AstAny::DirectiveAlign(ast_align) = any_node
        {
            if ast_align.item_ref.is_some() {
                continue;
            }

            let item_ref = defs.align_directives.next_item_ref();

            let res = AlignDirective {
                item_ref,
                align_size: 0,
                value: expr::Value::make_unknown(),
                resolved: false,
            };
            
            defs.align_directives.define(item_ref, res);
                
            ast_align.item_ref = Some(item_ref);
        }
    }


    Ok(())
}