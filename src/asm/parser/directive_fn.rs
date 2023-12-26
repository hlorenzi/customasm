use crate::*;

#[derive(Debug)]
pub struct AstDirectiveFn
{
    pub header_span: diagn::Span,
    pub name_span: diagn::Span,
    pub name: String,
    pub params: Vec<AstFnParameter>,
    pub body: expr::Expr,

    pub item_ref: Option<util::ItemRef<asm::Symbol>>,
}

#[derive(Debug)]
pub struct AstFnParameter
{
    pub name: String,
}

pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span,
) -> Result<AstDirectiveFn, ()>
{
    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.clone().unwrap();

    walker.expect(report, syntax::TokenKind::ParenOpen)?;

    let mut params = Vec::new();

    while !walker.is_over() && !walker.next_is(0, syntax::TokenKind::ParenClose)
    {
        let tk_param_name = walker.expect(report, syntax::TokenKind::Identifier)?;
        let param_name = tk_param_name.excerpt.clone().unwrap();

        params.push(AstFnParameter { name: param_name });

        walker.maybe_expect(syntax::TokenKind::Comma);
    }

    walker.expect(report, syntax::TokenKind::ParenClose)?;
    walker.expect(report, syntax::TokenKind::HeavyArrowRight)?;

    let body = expr::parse(report, walker)?;

    Ok(AstDirectiveFn {
        header_span,
        name_span: tk_name.span,
        name,
        params,
        body,

        item_ref: None,
    })
}
