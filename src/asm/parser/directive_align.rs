use crate::*;

#[derive(Debug)]
pub struct AstDirectiveAlign
{
    pub header_span: diagn::Span,
    pub expr: expr::Expr,

    pub item_ref: Option<util::ItemRef<asm::AlignDirective>>,
}

pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span,
) -> Result<AstDirectiveAlign, ()>
{
    let expr = expr::parse(report, walker)?;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveAlign {
        header_span,
        expr,

        item_ref: None,
    })
}
