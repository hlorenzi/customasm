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

            let provider = expr::EvalProvider {
                eval_var: &expr::dummy_eval_var(),
                eval_fn: &expr::dummy_eval_fn(),
                eval_asm: &expr::dummy_eval_asm(),
            };
            
            let addr_unit = match node.addr_unit
            {
                None => 8,
                Some(ref expr) => expr
                    .eval2(report, &provider)?
                    .expect_usize(report, &expr.span())?
            };

            println!("bankdef addr_unit = {}", addr_unit);

            /*let ruledef = Bankdef {
                item_ref,
            };

            defs.ruledefs.set(item_ref, ruledef);*/
        }
    }


    Ok(())
}