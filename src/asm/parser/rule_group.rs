use crate::*;


pub fn parse_directive_ruledef(
    state: &mut asm::parser::State,
    enable: bool)
    -> Result<(), ()>
{
    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.as_ref().unwrap().clone();

    state.parser.expect(syntax::TokenKind::BraceOpen)?;

    let mut rule_group = asm::RuleGroup
    {
        name: name.clone(),
        rules: Vec::new(),
    };

    while !state.parser.next_is(0, syntax::TokenKind::BraceClose)
    {
        rule_group.rules.push(asm::parser::parse_rule(state)?);
        state.parser.expect_linebreak()?;
    }

    state.parser.expect(syntax::TokenKind::BraceClose)?;

    state.asm_state.rule_groups.push(rule_group);

    if enable
    {
        state.asm_state.activate_rule_group(
            name,
            state.report.clone(),
            &tk_name.span)?;
    }

    Ok(())
}


pub fn parse_directive_enable(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.as_ref().unwrap().clone();

    state.asm_state.activate_rule_group(
        name,
        state.report.clone(),
        &tk_name.span)?;

    Ok(())
}