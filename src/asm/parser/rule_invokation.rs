use crate::*;


pub fn parse_rule_invokation(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let mut subparser = state.parser.slice_until_linebreak();
    subparser.suppress_reports();

    let candidates = match_active_rule_groups(state, &mut subparser)?;
    if candidates.len() != 0
    {
        let rule = state.asm_state.get_rule(candidates[0].rule_ref).unwrap();
        let production_size = rule.production.size().unwrap();
        let bit_offset = state.asm_state.banks[0].cur_bit_offset;
        state.asm_state.banks[0].rule_invokations.push(asm::RuleInvokation
        {
            bit_offset,
            candidates,
        });
        state.asm_state.banks[0].cur_bit_offset += production_size;
    }

    state.parser.expect_linebreak()?;

    Ok(())
}


pub fn match_active_rule_groups(
    state: &asm::parser::State,
    subparser: &mut syntax::Parser)
    -> Result<Vec<asm::RuleInvokationCandidate>, ()>
{
    let mut candidates = Vec::new();

    for rule_group_ref in &state.asm_state.active_rule_groups
    {
        let mut subparser_clone = subparser.clone();
        if let Ok(subcandidates) = match_rule_group(state, *rule_group_ref, &mut subparser_clone, true)
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


pub fn match_rule_group<'a>(
    state: &asm::parser::State,
    rule_group_ref: asm::RuleGroupRef,
    subparser: &mut syntax::Parser<'a>,
    must_consume_all_tokens: bool)
    -> Result<Vec<(asm::RuleInvokationCandidate, syntax::Parser<'a>)>, ()>
{
    let rule_group = &state.asm_state.rule_groups[rule_group_ref.index];

    let mut candidates = Vec::new();

    for index in 0..rule_group.rules.len()
    {
        let rule_ref = asm::RuleRef
        {
            rule_group_ref,
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
    let rule_group = &state.asm_state.rule_groups[rule_ref.rule_group_ref.index];
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

                        let expr = expr::Expression::parse(&mut expr_parser)?;
                        candidate.args.push(asm::RuleInvokationCandidateArgument::Expression(expr));

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

                    asm::PatternParameterType::RuleGroup(rule_group_ref)=>
                    {
                        //println!("> try match subrule {:?}", rule_group_ref);
                        
                        let subcandidates = match_rule_group(state, rule_group_ref, subparser, false)?;
                        if subcandidates.len() == 0
                        {
                            return Err(());
                        }

                        for subcandidate in &subcandidates[1..]
                        {
                            if subcandidate.1.get_current_token_index() != subcandidates[0].1.get_current_token_index()
                            {
                                state.report.error_span("ambiguous sub-rule parameter", &subparser.get_full_span());
                                return Err(());
                            }
                        }

                        //println!("> match!");
                        subparser.restore(subcandidates[0].1.save());
                        
                        let subcandidates = subcandidates.into_iter().map(|c| c.0).collect();
                        candidate.args.push(asm::RuleInvokationCandidateArgument::RuleGroup(subcandidates));
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