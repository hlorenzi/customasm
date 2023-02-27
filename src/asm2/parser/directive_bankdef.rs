use super::*;


#[derive(Debug)]
pub struct AstDirectiveBankdef
{
    pub header_span: diagn::Span,
    pub name: String,
    pub fields: AstFields,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span)
    -> Result<AstDirectiveBankdef, ()>
{
    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.clone().unwrap();

    walker.expect(report, syntax::TokenKind::BraceOpen)?;

    let fields = fields::parse(report, walker)?;
    fields::validate_names(
        report,
        &fields,
        &["addr", "size", "fill", "output", "bits", "labelalign"])?;

    walker.expect(report, syntax::TokenKind::BraceClose)?;
    walker.expect_linebreak(report)?;

    Ok(AstDirectiveBankdef {
        header_span,
        name,
        fields,
    })
}