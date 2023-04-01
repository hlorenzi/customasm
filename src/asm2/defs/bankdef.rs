use crate::*;


#[derive(Debug)]
pub struct Bankdef
{
    pub item_ref: util::ItemRef<Self>,
    pub addr_unit: usize,
    pub label_align: Option<usize>,
	pub addr_start: util::BigInt,
    pub size: Option<usize>,
	pub output_offset: Option<usize>,
	pub fill: bool,
}


pub fn define(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    _decls: &mut asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    let initial_item_ref = util::ItemRef::new(0);

    let initial_bankdef = Bankdef {
        item_ref: initial_item_ref,
        addr_unit: 8,
        label_align: None,
        addr_start: util::BigInt::new(0, None),
        size: None,
        output_offset: Some(0),
        fill: false,
    };

    defs.bankdefs.define(initial_item_ref, initial_bankdef);



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
                None => None,
                Some(expr) => Some(expr.eval_usize(report, &mut provider)?),
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
            
            let addr_end = match &node.addr_end
            {
                None => None,
                Some(expr) => Some(expr.eval_bigint(report, &mut provider)?),
            };

            let addr_size = {
                match (addr_size, addr_end)
                {
                    (None, None) => None,
                    (Some(size), None) => Some(size),
                    (None, Some(end)) =>
                    {
                        let maybe_size =
                            (&end - &addr_start).checked_to_usize();
                        
                        match maybe_size
                        {
                            Some(size) => Some(size),
                            None =>
                            {
                                report.error_span(
                                    "value out of supported range",
                                    node.addr_end.as_ref().unwrap().span());
        
                                return Err(());
                            }
                        }
                    }
                    (Some(_), Some(_)) =>
                    {
                        report.error_span(
                            "both `addr_end` and `size` defined",
                            &node.header_span);

                        return Err(());
                    }
                }
            };

            // FIXME: Multiplication can overflow
            let size = addr_size
                .map(|s| s * addr_unit);
            
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
                size,
                output_offset,
                fill,
            };

            defs.bankdefs.define(item_ref, bankdef);
        }
    }


    Ok(())
}