use crate::*;


#[derive(Debug)]
pub struct DataElement
{
    pub item_ref: util::ItemRef<Self>,
    pub position_within_bank: Option<usize>,
    pub elem_size: Option<usize>,
    pub encoding_statically_known: bool,
    pub encoding: expr::Value,
    pub resolved: bool,
}


pub fn define(
    _report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast: &mut asm::AstTopLevel,
    _decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm::AstAny::DirectiveData(ast_data) = any_node
        {
            for _ in &ast_data.elems
            {
                let item_ref = defs.data_elems.next_item_ref();

                let mut provider = expr::StaticallyKnownProvider::new(opts);
                provider.query_function = &asm::resolver::resolve_and_get_statically_known_builtin_fn;
                
                let data_block = DataElement {
                    item_ref,
                    position_within_bank: None,
                    elem_size: ast_data.elem_size,
                    encoding_statically_known: false,
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