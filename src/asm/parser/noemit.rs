use crate::*;

pub fn parse_directive_noemit(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_status = state.parser.expect(syntax::TokenKind::Identifier)?;
    let status = tk_status.excerpt.as_ref().unwrap().to_ascii_lowercase();

    match status.as_ref()
    {
        "on" => state.asm_state.is_noemit = true,
        "off" => state.asm_state.is_noemit = false,
        _ =>
        {
            state.report.error_span(
                "unknown noemit state",
                &tk_status.span,
            );
            return Err(());
        }
    }
    Ok(())
}