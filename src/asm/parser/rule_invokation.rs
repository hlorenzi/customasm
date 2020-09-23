use crate::*;


pub fn parse_rule_invokation(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let mut subparser = state.parser.slice_until_linebreak();
    subparser.suppress_reports();

    let candidates = match_active_rulesets(state, &mut subparser)?;
    if candidates.len() != 0
    {
        let mut invokation = asm::Invokation
        {
            ctx: state.asm_state.get_ctx(),
            size_guess: 0,
            span: subparser.get_full_span(),
            kind: asm::InvokationKind::Rule(asm::RuleInvokation
            {
                candidates,
            })
        };

        let resolved = state.asm_state.resolve_rule_invokation(
            state.report.clone(),
            &invokation,
            false);

        invokation.size_guess = match resolved
        {
            Ok(expr::Value::Integer(bigint)) =>
            {
                match bigint.size
                {
                    Some(size) => size,
                    None => 0,
                }
            }
            _ => 0
        };

        let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
        bankdata.push_invokation(invokation);
    }

    state.parser.expect_linebreak()?;

    Ok(())
}


pub fn match_active_rulesets(
    state: &asm::parser::State,
    subparser: &mut syntax::Parser)
    -> Result<Vec<asm::RuleInvokationCandidate>, ()>
{
    let mut candidates = Vec::new();

    for ruleset_ref in &state.asm_state.active_rulesets
    {
        let mut subparser_clone = subparser.clone();
        if let Ok(subcandidates) = match_ruleset(state, *ruleset_ref, &mut subparser_clone, true)
        {
            for candidate in subcandidates
            {
                candidates.push(candidate.0);
            }
        }
    }

    if candidates.len() == 0
    {
        state.report.error_span("no match for instruction found", &subparser.get_full_span());
    }

    //println!(
    //    "rule candidates for `{}`:\n{:#?}",
    //    state.fileserver.get_excerpt(&subparser.get_full_span()),
    //    candidates);

    Ok(candidates)
}


pub fn match_ruleset<'a>(
    state: &asm::parser::State,
    ruleset_ref: asm::RulesetRef,
    subparser: &mut syntax::Parser<'a>,
    must_consume_all_tokens: bool)
    -> Result<Vec<(asm::RuleInvokationCandidate, syntax::Parser<'a>)>, ()>
{
    let rule_group = &state.asm_state.rulesets[ruleset_ref.index];

    let mut candidates = Vec::new();

    for index in 0..rule_group.rules.len()
    {
        let rule_ref = asm::RuleRef
        {
            ruleset_ref,
            index,
        };

        let mut subparser_clone = subparser.clone();

        if let Ok(candidate) = match_rule(state, rule_ref, &mut subparser_clone)
        {
            //println!(
            //    "finish pattern with parser at `{}`",
            //    state.fileserver.get_excerpt(&subparser_clone.get_next_spans(10)));
        
            if !must_consume_all_tokens || subparser_clone.is_over()
            {
                candidates.push((candidate, subparser_clone));
            }
        }
    }

    Ok(candidates)
}


pub fn match_rule(
    state: &asm::parser::State,
    rule_ref: asm::RuleRef,
    subparser: &mut syntax::Parser)
    -> Result<asm::RuleInvokationCandidate, ()>
{
    let rule_group = &state.asm_state.rulesets[rule_ref.ruleset_ref.index];
    let rule = &rule_group.rules[rule_ref.index];

    let mut candidate = asm::RuleInvokationCandidate
    {
        rule_ref,
        args: Vec::new(),
    };
    
    //println!(
    //    "parse pattern with parser at `{}`",
    //    state.fileserver.get_excerpt(&subparser.get_next_spans(10)));

    for (index, part) in rule.pattern.iter().enumerate()
    {
        match part
        {
            asm::PatternPart::Exact(c) =>
            {
                //println!("> try match exact {}", c);

                if subparser.next_partial().to_ascii_lowercase() != *c
                {
                    return Err(());
                }

                //println!("> match!");
                subparser.advance_partial();
            }

            asm::PatternPart::Parameter(param_index) =>
            {
                let param = &rule.parameters[*param_index];

                match param.typ
                {
                    asm::PatternParameterType::Unspecified =>
                    {
                        //println!("> try match expr");
                        if subparser.is_at_partial()
                        {
                            match subparser.maybe_expect_partial_usize()
                            {
                                Some(value) =>
                                {
                                    let expr = expr::Value::make_integer(value).make_literal();
                                    candidate.args.push(asm::RuleInvokationArgument::Expression(expr));
                                }
                                None => return Err(())
                            }
                        }
                        else
                        {
                            let mut expr_parser = subparser.clone();
                            let mut expr_using_slice = false;

                            let next_part = rule.pattern.get(index + 1);

                            if let Some(asm::PatternPart::Exact(next_part_char)) = next_part
                            {
                                if let Some(slice_parser) = subparser.slice_until_char_or_nesting(*next_part_char)
                                {
                                    expr_parser = slice_parser;
                                    expr_using_slice = true;
                                }
                            }

                            //println!(
                            //    ">> parse argument expr with parser at `{}`",
                            //    state.fileserver.get_excerpt(&expr_parser.get_next_spans(10)));

                            let expr = expr::Expr::parse(&mut expr_parser)?;
                            candidate.args.push(asm::RuleInvokationArgument::Expression(expr));

                            if !expr_using_slice
                            {
                                subparser.restore(expr_parser.save());
                            }

                            //println!(
                            //    ">> continue with parser at {} = `{}`",
                            //    subparser.get_current_token_index(),
                            //    state.fileserver.get_excerpt(&subparser.get_next_spans(10)));
                                
                            //println!("> match!");
                        }
                    }

                    asm::PatternParameterType::RuleGroup(rule_group_ref)=>
                    {
                        //println!("> try match subrule {:?}", rule_group_ref);
                        
                        let subcandidates = match_ruleset(state, rule_group_ref, subparser, false)?;
                        if subcandidates.len() == 0
                        {
                            return Err(());
                        }

                        for subcandidate in &subcandidates[1..]
                        {
                            if subcandidate.1.get_current_token_index() != subcandidates[0].1.get_current_token_index()
                            {
                                state.report.error_span("ambiguous nested ruleset", &subparser.get_full_span());
                                return Err(());
                            }
                        }

                        //println!("> match!");
                        subparser.restore(subcandidates[0].1.save());
                        
                        let subcandidates = subcandidates.into_iter().map(|c| c.0).collect();
                        candidate.args.push(asm::RuleInvokationArgument::NestedRuleset(subcandidates));
                    }
                }
            }
        }
    }

    if subparser.is_at_partial()
    {
        return Err(());
    }

    Ok(candidate)
}