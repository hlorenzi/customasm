use crate::*;


#[derive(Debug)]
pub struct AstInstruction
{
    pub span: diagn::Span,
    pub tokens: Vec<syntax::Token>,

    pub item_ref: Option<util::ItemRef<asm2::Instruction>>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker)
    -> Result<AstInstruction, ()>
{
    let cutoff_walker = walker
        .cutoff_at_linebreak_while_respecting_braces();

    walker.expect_linebreak(report)?;
    
    Ok(AstInstruction {
        span: cutoff_walker.get_full_span(),
        tokens: cutoff_walker.get_cloned_tokens(),

        item_ref: None,
    })
}