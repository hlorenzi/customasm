use super::*;


#[derive(Debug)]
pub struct AstDirectiveRes
{
    pub header_span: diagn::Span,
    pub expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span)
    -> Result<AstDirectiveRes, ()>
{
    let expr = expr::parse(report, walker)?;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveRes {
        header_span,
        expr,
    })
}