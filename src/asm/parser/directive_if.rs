use crate::{asm::AstTopLevel, *};

#[derive(Debug)]
pub struct AstDirectiveIf
{
    pub header_span: diagn::Span,
    pub condition_expr: expr::Expr,

    pub true_arm: asm::AstTopLevel,
    pub false_arm: Option<asm::AstTopLevel>,
}

pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span,
) -> Result<AstDirectiveIf, ()>
{
    let expr = expr::parse(report, walker)?;

    let true_arm = parse_braced_block(report, walker)?;
    let false_arm = parse_else_blocks(report, walker)?;

    Ok(AstDirectiveIf {
        header_span,
        condition_expr: expr,

        true_arm,
        false_arm,
    })
}

fn parse_braced_block(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
) -> Result<AstTopLevel, ()>
{
    walker.expect(report, syntax::TokenKind::BraceOpen)?;

    let block = asm::parser::parse_nested_toplevel(report, walker)?;

    walker.expect(report, syntax::TokenKind::BraceClose)?;

    Ok(block)
}

fn parse_else_blocks(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
) -> Result<Option<AstTopLevel>, ()>
{
    if !walker.next_is(0, syntax::TokenKind::Hash)
        || !walker.next_is(1, syntax::TokenKind::Identifier)
    {
        return Ok(None);
    }

    let directive_name = walker
        .next_nth(1)
        .excerpt
        .as_ref()
        .map(|s| s.as_str())
        .unwrap();

    if directive_name == "else"
    {
        walker.expect(report, syntax::TokenKind::Hash)?;

        walker.expect(report, syntax::TokenKind::Identifier)?;

        Ok(Some(parse_braced_block(report, walker)?))
    }
    else if directive_name == "elif"
    {
        let tk_hash = walker.expect(report, syntax::TokenKind::Hash)?;

        let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;

        let header_span = tk_hash.span.join(tk_name.span);

        let ast_if = parse(report, walker, header_span)?;

        let ast_toplevel = AstTopLevel {
            nodes: vec![asm::AstAny::DirectiveIf(ast_if)],
        };

        Ok(Some(ast_toplevel))
    }
    else
    {
        Ok(None)
    }
}
