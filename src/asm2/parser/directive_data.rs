use super::*;


#[derive(Debug)]
pub struct AstDirectiveData
{
    pub header_span: diagn::Span,
    pub elem_size: Option<usize>,
    pub elems: Vec<expr::Expr>,

    pub item_refs: Vec<util::ItemRef<asm2::DataElement>>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    elem_size: Option<usize>,
    header_span: diagn::Span)
    -> Result<AstDirectiveData, ()>
{
    let mut elems = Vec::new();

    loop
    {
        elems.push(expr::parse(report, walker)?);

        if !walker.maybe_expect(syntax::TokenKind::Comma).is_some()
        {
            break;
        }

        if walker.next_is_linebreak()
        {
            break;
        }
    }

    walker.expect_linebreak(report)?;

    Ok(AstDirectiveData {
        header_span,
        elem_size,
        elems,

        item_refs: Vec::new(),
    })
}