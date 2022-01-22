use crate::*;


pub fn parse_directive_fn(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.unwrap().clone();

    if let Some(duplicate) = state.asm_state.functions.iter().find(|r| r.name == name)
    {
        let _guard = state.report.push_parent("duplicate function", &tk_name.span);
        state.report.note_span("first declared here", &duplicate.decl_span);
        return Err(());
    }

    state.parser.expect(syntax::TokenKind::ParenOpen)?;

    let mut params = Vec::new();
    while !state.parser.is_over() && !state.parser.next_is(0, syntax::TokenKind::ParenClose)
    {
        let tk_param = state.parser.expect(syntax::TokenKind::Identifier)?;
        let param = tk_param.excerpt.unwrap().clone();
        params.push(param);

        state.parser.maybe_expect(syntax::TokenKind::Comma);
    }

    state.parser.expect(syntax::TokenKind::ParenClose)?;

    state.parser.expect(syntax::TokenKind::HeavyArrowRight)?;

    let body = expr::Expr::parse(&mut state.parser)?;

    let function = asm::Function
    {
        decl_span: tk_name.span.clone(),
        name,
        params,
        body,
    };

    state.asm_state.functions.push(function);

    Ok(())
}