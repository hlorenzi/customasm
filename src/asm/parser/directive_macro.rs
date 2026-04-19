use crate::*;


#[derive(Clone, Debug)]
pub struct AstDirectiveMacro
{
    pub header_span: diagn::Span,
    pub pattern: asm::AstRulePattern,
    pub body: asm::AstTopLevel,

    pub item_ref: Option<util::ItemRef::<asm::Ruledef>>,
}


pub fn parse(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    walker: &mut syntax::Walker,
    header_span: diagn::Span)
    -> Result<AstDirectiveMacro, ()>
{
    let pattern = asm::parser::parse_pattern(
        report,
        opts,
        walker,
        false)?;

    walker.expect(report, syntax::TokenKind::BraceOpen)?;

    let body = asm::parser::parse_nested_toplevel(
        report,
        opts,
        walker)?;

    walker.expect(report, syntax::TokenKind::BraceClose)?;
    walker.expect_linebreak(report)?;

    Ok(AstDirectiveMacro {
        header_span,
        pattern,
        body,
        item_ref: None,
    })
}