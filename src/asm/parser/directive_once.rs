use crate::*;


#[derive(Debug)]
pub struct AstDirectiveOnce
{
    pub header_span: diagn::Span,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span)
    -> Result<AstDirectiveOnce, ()>
{
    walker.expect_linebreak(report)?;

    Ok(AstDirectiveOnce {
        header_span,
    })
}