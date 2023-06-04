use crate::*;


#[derive(Debug)]
pub struct Function
{
    pub item_ref: util::ItemRef<asm::Symbol>,
    pub fn_ref: util::ItemRef<Self>,
    pub params: Vec<FunctionParameter>,
    pub body: expr::Expr,
}


#[derive(Debug)]
pub struct FunctionParameter
{
    pub name: String,
}


pub fn define(
    _report: &mut diagn::Report,
    ast: &asm::AstTopLevel,
    _decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        if let asm::AstAny::DirectiveFn(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();

            let mut params = Vec::new();
            for param in &node.params
            {
                params.push(FunctionParameter {
                    name: param.name.clone(),
                });
            }

            let body = node.body.clone();

            let fn_ref = defs.functions.next_item_ref();

            let function = Function {
                item_ref,
                fn_ref,
                params,
                body,
            };

            let symbol = asm::Symbol {
                item_ref,
                no_emit: true,
                value_statically_known: true,
                value: expr::Value::Function(fn_ref.0),
                resolved: true,
                bankdef_ref: None,
            };

            defs.functions.define(fn_ref, function);
            defs.symbols.define(item_ref, symbol);
        }
    }


    Ok(())
}