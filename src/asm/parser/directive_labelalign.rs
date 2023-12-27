use crate::*;


#[derive(Clone, Debug)]
pub struct AstDirectiveLabelAlign
{
    pub header_span: diagn::Span,
    pub expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    _walker: &mut syntax::Walker,
    header_span: diagn::Span)
    -> Result<AstDirectiveLabelAlign, ()>
{
    report.error_span(
        "standalone `#labelalign` is deprecated; use it inside a `#bankdef`",
        header_span);
    
    Err(())
}