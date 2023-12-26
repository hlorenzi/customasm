use crate::*;

#[derive(Debug)]
pub struct AstDirectiveNoEmit
{
    pub header_span: diagn::Span,
    pub status: bool,
}

pub fn parse(
    report: &mut diagn::Report,
    _walker: &mut syntax::TokenWalker,
    header_span: diagn::Span,
) -> Result<AstDirectiveNoEmit, ()>
{
    report.error_span(
        "`#noemit` is deprecated; use `#const(noemit)` at each constant declaration",
        header_span,
    );

    Err(())
}
