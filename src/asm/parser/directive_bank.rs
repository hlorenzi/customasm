use crate::*;

#[derive(Debug)]
pub struct AstDirectiveBank
{
    pub header_span: diagn::Span,
    pub name_span: diagn::Span,
    pub name: String,

    pub item_ref: Option<util::ItemRef<asm::Bankdef>>,
}

pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span,
) -> Result<AstDirectiveBank, ()>
{
    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.clone().unwrap();
    let name_span = tk_name.span;

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveBank {
        header_span,
        name_span,
        name,

        item_ref: None,
    })
}
