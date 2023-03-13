use super::*;


#[derive(Debug)]
pub struct AstInstruction
{
    pub tokens: Vec<syntax::Token>,

    pub matches: asm2::InstructionMatches,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker)
    -> Result<AstInstruction, ()>
{
    let tokens = walker
        .slice_until_linebreak_over_nested_braces()
        .get_cloned_tokens();

    walker.expect_linebreak(report)?;
    
    Ok(AstInstruction {
        tokens,
        matches: asm2::InstructionMatches::new(),
    })
}