use crate::*;


pub fn parse_directive_bankdef(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.as_ref().unwrap().clone();

    state.parser.expect(syntax::TokenKind::BraceOpen)?;

    let mut bank = asm::Bank
    {
        name: name.clone(),
        addr: util::BigInt::from(0),
        size: None,
        output_offset: Some(0),
        fill: false,
        decl_span: None,
    };

    while !state.parser.next_is(0, syntax::TokenKind::BraceClose)
    {
        parse_bankdef_field(state, &mut bank)?;
        state.parser.expect_linebreak_or(syntax::TokenKind::Comma)?;
    }

    state.parser.expect(syntax::TokenKind::BraceClose)?;

    state.asm_state.create_bank(bank);

    Ok(())
}


fn parse_bankdef_field(
    state: &mut asm::parser::State,
    bank: &mut asm::Bank)
    -> Result<(), ()>
{
    let _tk_hash = state.parser.expect(syntax::TokenKind::Hash)?;
    let tk_field_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let field_name = tk_field_name.excerpt.as_ref().unwrap().clone();

    match field_name.as_ref()
    {
        "addr" =>
            bank.addr = asm::parser::parse_expr_bigint(state)?,
            
        "outp" => bank.output_offset =
            Some(asm::parser::parse_expr_usize_fn(state, |u| u.checked_mul(8))?),

        _ =>
        {
            state.report.error_span("unknown bankdef field", &tk_field_name.span);
            return Err(());
        }
    }

    Ok(())
}


pub fn parse_directive_bank(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.as_ref().unwrap().clone();

    state.asm_state.cur_bank = state.asm_state.find_bank(
        name,
        state.report.clone(),
        &tk_name.span)?;

    Ok(())
}