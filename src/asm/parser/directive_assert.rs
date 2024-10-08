use crate::*;


#[derive(Clone, Debug)]
pub struct AstDirectiveAssert
{
    pub header_span: diagn::Span,
    pub condition_expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker,
    header_span: diagn::Span)
    -> Result<AstDirectiveAssert, ()>
{
    let expr = expr::parse(report, walker)?;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveAssert {
        header_span,
        condition_expr: expr,
    })
}