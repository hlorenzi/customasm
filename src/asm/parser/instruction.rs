use crate::*;


#[derive(Clone, Debug)]
pub struct AstInstruction
{
    pub span: diagn::Span,
    pub src: String,

    pub item_ref: Option<util::ItemRef<asm::Instruction>>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker)
    -> Result<AstInstruction, ()>
{
    walker.skip_ignorable();
    
    let line = walker.advance_until_linebreak();

    walker.expect_linebreak(report)?;
    
    Ok(AstInstruction {
        span: line.get_full_span(),
        src: line.get_full_excerpt().to_string(),

        item_ref: None,
    })
}