use crate::*;


#[derive(Clone, Debug)]
pub struct AstDirectiveAddr
{
    pub header_span: diagn::Span,
    pub expr: expr::Expr,

    pub item_ref: Option<util::ItemRef<asm::AddrDirective>>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker,
    header_span: diagn::Span)
    -> Result<AstDirectiveAddr, ()>
{
    let expr = expr::parse(report, walker)?;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveAddr {
        header_span,
        expr,

        item_ref: None,
    })
}