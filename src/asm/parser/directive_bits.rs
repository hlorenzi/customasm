use crate::*;

#[derive(Debug)]
pub struct AstDirectiveBits
{
    pub header_span: diagn::Span,
    pub expr: expr::Expr,
}

pub fn parse(
    report: &mut diagn::Report,
    _walker: &mut syntax::TokenWalker,
    header_span: diagn::Span,
) -> Result<AstDirectiveBits, ()>
{
    report.error_span(
        "standalone `#bits` is deprecated; use it inside a `#bankdef`",
        header_span,
    );

    Err(())
}
