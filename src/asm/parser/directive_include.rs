use crate::*;

#[derive(Debug)]
pub struct AstDirectiveInclude
{
    pub header_span: diagn::Span,
    pub filename_span: diagn::Span,
    pub filename: String,
}

pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span,
) -> Result<AstDirectiveInclude, ()>
{
    let tk_filename = walker.expect(report, syntax::TokenKind::String)?;

    let filename = syntax::excerpt_as_string_contents(
        report,
        tk_filename.span,
        tk_filename.excerpt.as_ref().unwrap(),
    )?;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveInclude {
        header_span: header_span.join(tk_filename.span),
        filename_span: tk_filename.span,
        filename,
    })
}
