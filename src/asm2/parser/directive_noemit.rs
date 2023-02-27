use super::*;


#[derive(Debug)]
pub struct AstDirectiveNoEmit
{
    pub header_span: diagn::Span,
    pub status: bool,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span)
    -> Result<AstDirectiveNoEmit, ()>
{
    let tk_status = walker.expect(report, syntax::TokenKind::Identifier)?;
    let status = tk_status.excerpt.as_ref().unwrap().to_ascii_lowercase();

    walker.expect_linebreak(report)?;

    let status = match status.as_ref()
    {
        "on" => true,
        "off" => false,
        _ =>
        {
            report.error_span(
                "unknown noemit state",
                &tk_status.span,
            );

            return Err(());
        }
    };

    Ok(AstDirectiveNoEmit {
        header_span,
        status,
    })
}