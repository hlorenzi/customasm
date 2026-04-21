use crate::*;


#[derive(Debug)]
pub struct DataElement
{
    pub item_ref: util::ItemRef<Self>,
    pub elem_size: Option<usize>,
    pub encoding: expr::Value,
    pub resolved: bool,
}


pub fn define(
    _report: &mut diagn::Report,
    _opts: &asm::AssemblyOptions,
    ast: &mut asm::AstTopLevel,
    _decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm::AstAny::DirectiveData(ast_data) = any_node
        {
            if ast_data.item_refs.len() == ast_data.elems.len() {
                continue;
            }

            for _ in &ast_data.elems
            {
                let item_ref = defs.data_elems.next_item_ref();

                let data_block = DataElement {
                    item_ref,
                    elem_size: ast_data.elem_size,
                    encoding: expr::Value::make_unknown(),
                    resolved: false,
                };
                
                defs.data_elems.define(item_ref, data_block);
                    
                ast_data.item_refs.push(item_ref);
            }
        }
    }


    Ok(())
}