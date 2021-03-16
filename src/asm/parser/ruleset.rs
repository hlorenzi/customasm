use crate::*;


pub fn parse_directive_ruledef(
    state: &mut asm::parser::State,
    tk_directive: &syntax::Token,
    is_not_subruledef: bool)
    -> Result<(), ()>
{
    let mut decl_span = tk_directive.span.clone();
    let mut name = format!("{}_anonymous", state.asm_state.rulesets.len());

    if let Some(tk_name) = state.parser.maybe_expect(syntax::TokenKind::Identifier)
    {
        decl_span = tk_name.span.clone();
        name = tk_name.excerpt.as_ref().unwrap().clone();

        if let Some(duplicate) = state.asm_state.rulesets.iter().find(|r| r.name == name)
        {
            let _guard = state.report.push_parent("duplicate ruleset", &tk_name.span);
            state.report.note_span("first declared here", &duplicate.decl_span);
            return Err(());
        }
    }

    state.parser.expect(syntax::TokenKind::BraceOpen)?;

    let mut ruleset = asm::Ruleset
    {
        name: name.clone(),
        rules: Vec::new(),
        decl_span: decl_span.clone(),
    };

    while !state.parser.next_is(0, syntax::TokenKind::BraceClose)
    {
        ruleset.rules.push(asm::parser::parse_rule(state, is_not_subruledef)?);
        state.parser.expect_linebreak()?;
    }

    state.parser.expect(syntax::TokenKind::BraceClose)?;

    state.asm_state.rulesets.push(ruleset);

    if is_not_subruledef
    {
        state.asm_state.activate_ruleset(
            name,
            state.report.clone(),
            &decl_span)?;
    }

    Ok(())
}


pub fn parse_directive_enable(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.as_ref().unwrap().clone();

    state.asm_state.activate_ruleset(
        name,
        state.report.clone(),
        &tk_name.span)?;

    Ok(())
}