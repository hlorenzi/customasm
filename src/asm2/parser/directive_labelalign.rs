use super::*;


#[derive(Debug)]
pub struct AstDirectiveLabelAlign
{
    pub header_span: diagn::Span,
    pub expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    _walker: &mut syntax::TokenWalker,
    header_span: diagn::Span)
    -> Result<AstDirectiveLabelAlign, ()>
{
    report.error_span(
        "standalone `#labelalign` is deprecated; use it inside a `#bankdef`",
        &header_span);
    
    Err(())
    /*let expr = expr::parse(report, walker)?;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveLabelAlign {
        header_span,
        expr,
    })*/
}