use crate::*;


#[derive(Clone, Debug)]
pub struct AstDirectiveIf
{
    pub header_span: diagn::Span,
    pub condition_expr: expr::Expr,

    pub true_arm: asm::AstTopLevel,
    pub false_arm: Option<asm::AstTopLevel>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker,
    header_span: diagn::Span)
    -> Result<AstDirectiveIf, ()>
{
    let expr = expr::parse(report, walker)?;

    let true_arm = asm::parser::parse_braced_toplevel(report, walker)?;
    let false_arm = parse_else_blocks(report, walker)?;

    Ok(AstDirectiveIf {
        header_span,
        condition_expr: expr,

        true_arm,
        false_arm,
    })
}


fn parse_else_blocks(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker)
    -> Result<Option<asm::AstTopLevel>, ()>
{
    if !walker.next_useful_is(0, syntax::TokenKind::Hash) ||
        !walker.next_useful_is(1, syntax::TokenKind::Identifier)
    {
        return Ok(None);
    }

    let token = walker.next_nth_useful_token(1);
    let directive_name = walker.get_span_excerpt(token.span);

    if directive_name == "else"
    {
        walker.expect(
            report,
            syntax::TokenKind::Hash)?;
        
        walker.expect(
            report,
            syntax::TokenKind::Identifier)?;
        
        Ok(Some(asm::parser::parse_braced_toplevel(report, walker)?))
    }
    else if directive_name == "elif"
    {
        let tk_hash = walker.expect(
            report,
            syntax::TokenKind::Hash)?;

        let tk_name = walker.expect(
            report,
            syntax::TokenKind::Identifier)?;

        let header_span = tk_hash.span.join(tk_name.span);
        
        let ast_if = parse(
            report,
            walker,
            header_span)?;

        let ast_toplevel = asm::AstTopLevel {
            nodes: vec![asm::AstAny::DirectiveIf(ast_if)],
        };

        Ok(Some(ast_toplevel))
    }
    else
    {
        Ok(None)
    }
}