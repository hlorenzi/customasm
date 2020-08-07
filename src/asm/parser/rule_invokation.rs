use crate::*;


pub fn parse_rule_invokation(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let start_tk_index = state.parser.get_current_token_index();
    let mut end_tk_index = start_tk_index;
    while !state.parser.is_over() && !state.parser.next_is_linebreak()
    {
        state.parser.advance();
        end_tk_index = state.parser.get_previous_token_index() + 1;
    }

    println!("{} {}", start_tk_index, end_tk_index);

    state.parser.expect_linebreak()?;

    let mut subparser = state.parser.clone_slice(start_tk_index, end_tk_index);
    subparser.suppress_reports();

    let matched_rules = match_rules(state, &mut subparser)?;

    Ok(())
}


pub fn match_rules(
    state: &asm::parser::State,
    subparser: &mut syntax::Parser)
    -> Result<(), ()>
{
    let mut matched_rules = Vec::new();

    for active_rule_group_name in &state.asm_state.active_rule_groups
    {
        let rule_group = state.asm_state.rule_groups.get(active_rule_group_name).unwrap();

        for rule in &rule_group.rules
        {
            if let Ok(()) = check_rule(state, rule, &mut subparser.clone())
            {
                matched_rules.push(rule);
                println!("matched rule {:?}", rule);
            }
        }
    }

    if matched_rules.len() == 0
    {
        state.report.error_span("no match for instruction found", &subparser.get_full_span());
    }

    Ok(())
}


pub fn check_rule(
    state: &asm::parser::State,
    rule: &asm::Rule,
    subparser: &mut syntax::Parser)
    -> Result<(), ()>
{
    for part in &rule.pattern
    {
        match part
        {
            asm::PatternPart::Exact(c) =>
            {
                if subparser.next_partial() != *c
                {
                    return Err(());
                }

                subparser.advance_partial();
            }

            asm::PatternPart::Parameter(param_index) =>
            {
                let param = &rule.parameters[*param_index];

                match param.typ
                {
                    asm::PatternParameterType::Unspecified =>
                    {
                        let expr = expr::Expression::parse(subparser)?;
                    }

                    asm::PatternParameterType::RuleGroup{ ref name } =>
                    {
                        let rule_group = state.asm_state.rule_groups.get(name).unwrap();
                
                        for subrule in &rule_group.rules
                        {
                            if let Err(()) = check_rule(state, subrule, &mut subparser.clone())
                            {
                                return Err(());
                            }
                        }
                    }
                }
            }
        }
    }

    if subparser.is_at_partial()
    {
        return Err(());
    }

    Ok(())
}