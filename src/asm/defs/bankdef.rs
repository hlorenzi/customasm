use crate::*;


#[derive(Debug)]
pub struct Bankdef
{
    pub item_ref: util::ItemRef<Self>,
    pub addr_unit: usize,
    pub label_align: Option<usize>,
	pub addr_start: util::BigInt,
    pub size_in_units: Option<usize>,
    pub size_in_bits: Option<usize>,
	pub output_offset: Option<usize>,
	pub fill: bool,
	pub userdata: expr::Value,
}


pub fn define(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast: &asm::AstTopLevel,
    decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    let initial_item_ref = util::ItemRef::new(0);

    let initial_bankdef = Bankdef {
        item_ref: initial_item_ref,
        addr_unit: 8,
        label_align: None,
        addr_start: util::BigInt::new(0, None),
        size_in_units: None,
        size_in_bits: None,
        output_offset: Some(0),
        fill: false,
        userdata: expr::Value::make_void(),
    };

    defs.bankdefs.define(initial_item_ref, initial_bankdef);



    for any_node in &ast.nodes
    {
        if let asm::AstAny::DirectiveBankdef(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();

            let addr_unit = match &node.addr_unit
            {
                None => 8,
                Some(expr) =>
                    asm::resolver::eval_certain(
                        report,
                        opts,
                        decls,
                        defs,
                        expr)?
                    .expect_usize(report, expr.span())?,
            };
            
            let label_align = match &node.label_align
            {
                None => None,
                Some(expr) => Some(
                    asm::resolver::eval_certain(
                        report,
                        opts,
                        decls,
                        defs,
                        expr)?
                    .expect_usize(report, expr.span())?),
            };
            
            let addr_start = match &node.addr_start
            {
                None => util::BigInt::new(0, None),
                Some(expr) =>
                    asm::resolver::eval_certain(
                        report,
                        opts,
                        decls,
                        defs,
                        expr)?
                    .expect_bigint(report, expr.span())?
                    .clone(),
            };
            
            let addr_size = match &node.addr_size
            {
                None => None,
                Some(expr) => Some(
                    asm::resolver::eval_certain(
                        report,
                        opts,
                        decls,
                        defs,
                        expr)?
                    .expect_usize(report, expr.span())?),
            };
            
            let addr_end = match &node.addr_end
            {
                None => None,
                Some(expr) => Some(
                    asm::resolver::eval_certain(
                        report,
                        opts,
                        decls,
                        defs,
                        expr)?
                    .expect_bigint(report, expr.span())?
                    .clone()),
            };

            let addr_size = {
                match (addr_size, addr_end)
                {
                    (None, None) => None,
                    (Some(size), None) => Some(size),
                    (None, Some(end)) =>
                    {
                        Some(end
                            .checked_sub(
                                report,
                                node.addr_end.as_ref().unwrap().span(),
                                &addr_start)?
                            .checked_into::<usize>(
                                report,
                                node.addr_end.as_ref().unwrap().span())?)
                    }
                    (Some(_), Some(_)) =>
                    {
                        report.error_span(
                            "both `addr_end` and `size` defined",
                            node.header_span);

                        return Err(());
                    }
                }
            };

            let size_in_units = addr_size;
            
            let size_in_bits = {
                if let Some(addr_size) = addr_size
                {
                    match addr_size.checked_mul(addr_unit)
                    {
                        Some(s) => Some(s),
                        None =>
                        {
                            report.error_span(
                                "value is out of supported range",
                                node.addr_size.as_ref().unwrap().span());

                            return Err(());
                        }
                    }
                }
                else { None }
            };
            
            let output_offset = match &node.output_offset
            {
                None => None,
                Some(expr) => Some(
                    asm::resolver::eval_certain(
                        report,
                        opts,
                        decls,
                        defs,
                        expr)?
                    .expect_usize(report, expr.span())?),
            };

            let fill = node.fill;
            
            let userdata = match &node.userdata {
                None => expr::Value::make_void(),
                Some(expr) => asm::resolver::eval_certain(
                    report,
                    opts,
                    decls,
                    defs,
                    expr)?,
            };

            let bankdef = Bankdef {
                item_ref,
                addr_unit,
                label_align,
                addr_start,
                size_in_units,
                size_in_bits,
                output_offset,
                fill,
                userdata,
            };

            defs.bankdefs.define(item_ref, bankdef);
        }
    }


    Ok(())
}