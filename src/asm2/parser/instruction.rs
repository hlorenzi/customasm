use super::*;


#[derive(Debug)]
pub struct AstInstruction
{
    pub tokens: Vec<syntax::Token>,
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
    })
}