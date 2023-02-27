use super::*;


#[derive(Debug)]
pub struct AstDirectiveBits
{
    pub header_span: diagn::Span,
    pub expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span)
    -> Result<AstDirectiveBits, ()>
{
    let expr = expr::parse(report, walker)?;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveBits {
        header_span,
        expr,
    })
}