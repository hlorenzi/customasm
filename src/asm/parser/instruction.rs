use crate::*;

#[derive(Debug)]
pub struct AstInstruction
{
    pub span: diagn::Span,
    pub tokens: Vec<syntax::Token>,

    pub item_ref: Option<util::ItemRef<asm::Instruction>>,
}

pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
) -> Result<AstInstruction, ()>
{
    let skipped = walker.skip_until_linebreak_over_nested_braces();

    walker.expect_linebreak(report)?;

    Ok(AstInstruction {
        span: skipped.get_full_span(),
        tokens: skipped.get_cloned_tokens(),

        item_ref: None,
    })
}
