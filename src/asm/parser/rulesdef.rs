use crate::*;


pub fn parse_directive_rulesdef(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    
    state.parser.expect(syntax::TokenKind::BraceOpen)?;

    let mut rules = Vec::new();

    while !state.parser.next_is(0, syntax::TokenKind::BraceClose)
    {
        rules.push(asm::parse_rule(state)?);
        state.parser.expect_linebreak()?;
    }

    state.parser.expect(syntax::TokenKind::BraceClose)?;

    Ok(())
}