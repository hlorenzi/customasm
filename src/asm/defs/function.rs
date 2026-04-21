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
        if let asm::AstAny::DirectiveFn(ast_fn) = any_node
        {
            let item_ref = ast_fn.item_ref.unwrap();

            if defs.symbols.is_defined(item_ref) {
                continue;
            }

            let mut params = Vec::new();
            for param in &ast_fn.params
            {
                params.push(FunctionParameter {
                    name: param.name.clone(),
                });
            }

            let body = ast_fn.body.clone();

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
                value: expr::Value::Function(
                        expr::ValueMetadata::new(),
                        fn_ref)
                    .statically_known(),
                resolved: true,
                driver_defined: false,
                bankdef_ref: None,
            };

            defs.functions.define(fn_ref, function);
            defs.symbols.define(item_ref, symbol);
        }
    }


    Ok(())
}