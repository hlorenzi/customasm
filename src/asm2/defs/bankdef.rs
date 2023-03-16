use crate::*;


#[derive(Debug)]
pub struct Bankdef
{
    pub item_ref: util::ItemRef<Self>,
    pub addr_unit: usize,
    pub label_align: usize,
	pub addr_start: util::BigInt,
	pub addr_size: Option<usize>,
	pub output_offset: Option<usize>,
	pub fill: bool,
}


pub fn resolve(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        if let asm2::AstAny::DirectiveBankdef(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();

            let mut provider = expr::EvalProvider {
                eval_var: &mut expr::dummy_eval_var(),
                eval_fn: &mut expr::dummy_eval_fn(),
                eval_asm: &mut expr::dummy_eval_asm(),
            };
            
            let addr_unit = match &node.addr_unit
            {
                None => 8,
                Some(expr) => expr.eval_usize(report, &mut provider)?,
            };
            
            let label_align = match &node.label_align
            {
                None => 0,
                Some(expr) => expr.eval_usize(report, &mut provider)?,
            };
            
            let addr_start = match &node.addr_start
            {
                None => util::BigInt::new(0, None),
                Some(expr) => expr.eval_bigint(report, &mut provider)?,
            };
            
            let addr_size = match &node.addr_size
            {
                None => None,
                Some(expr) => Some(expr.eval_usize(report, &mut provider)?),
            };
            
            let output_offset = match &node.output_offset
            {
                None => None,
                Some(expr) => Some(expr.eval_usize(report, &mut provider)?),
            };

            let fill = node.fill;

            let bankdef = Bankdef {
                item_ref,
                addr_unit,
                label_align,
                addr_start,
                addr_size,
                output_offset,
                fill,
            };

            defs.bankdefs.define(item_ref, bankdef);
        }
    }


    Ok(())
}