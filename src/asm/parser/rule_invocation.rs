use crate::*;


static DEBUG: bool = false;


pub fn parse_rule_invocation(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let mut subparser = state.parser.slice_until_linebreak();
    subparser.suppress_reports();

    if DEBUG
    {
        println!("");
        println!(
            "=== parse rule invocation `{}` ===",
            state.fileserver.get_excerpt(&subparser.get_full_span()));
    }

    let mut candidates = match_active_rulesets(state, &mut subparser)?;
    if candidates.len() != 0
    {
        // Calculate specificity scores
        for candidate in &mut candidates
        {
            candidate.specificity = candidate.calculate_specificity_score(&state.asm_state);
        }

        // Sort candidates by specificity score
        candidates.sort_by(|a, b| b.specificity.cmp(&a.specificity));
        
        if DEBUG
        {
            println!("");
            println!("final candidates:");
            for candidate in &candidates
            {
                let rule_group = &state.asm_state.rulesets[candidate.rule_ref.ruleset_ref.index];
                let rule = &rule_group.rules[candidate.rule_ref.index];

                println!(
                    "  `{}`",
                    state.fileserver.get_excerpt(&rule.span));
            }
        }

        // Only keep candidates with the maximum specificity score
        let mut max_specificity = candidates[0].specificity;
        for candidate in &candidates[1..]
        {
            max_specificity = std::cmp::max(max_specificity, candidate.specificity);
        }

        candidates.retain(|c| c.specificity == max_specificity);

        let mut invocation = asm::Invocation
        {
            ctx: state.asm_state.get_ctx(&state),
            size_guess: 0,
            span: subparser.get_full_span(),
            kind: asm::InvocationKind::Rule(asm::RuleInvocation
            {
                candidates,
            })
        };
        
        let resolved = state.asm_state.resolve_rule_invocation(
            state.report.clone(),
            &invocation,
            state.fileserver,
            false);

        //println!("{} = {:?}", state.fileserver.get_excerpt(&invocation.span), &resolved);

        // TODO: can provide an exact guess even if resolution fails,
        // if we have an exact candidate, and
        // if the production expression returns a sized value
        invocation.size_guess = match resolved
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

        //println!("{} = {}", state.fileserver.get_excerpt(&invocation.span), invocation.size_guess);

        let bankdata = state.asm_state.get_bankdata(state.asm_state.cur_bank);
        bankdata.check_writable(&state.asm_state, state.report.clone(), &invocation.span)?;
        
        let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
        bankdata.push_invocation(invocation);
    }

    state.parser.expect_linebreak()?;

    Ok(())
}


pub fn match_active_rulesets(
    state: &asm::parser::State,
    subparser: &mut syntax::Parser)
    -> Result<Vec<asm::RuleInvocationCandidate>, ()>
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
    -> Result<Vec<(asm::RuleInvocationCandidate, syntax::Parser<'a>)>, ()>
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

        if let Ok(subcandidates) = match_rule(state, rule_ref, &mut subparser_clone)
        {
            //println!(
            //    "finish pattern with parser at `{}`",
            //    state.fileserver.get_excerpt(&subparser_clone.get_next_spans(10)));
        
            if !must_consume_all_tokens || subparser_clone.is_over()
            {
                for subcandidate in subcandidates
                {
                    candidates.push((subcandidate, subparser_clone.clone()));
                }
            }
        }
    }

    Ok(candidates)
}


pub fn match_rule(
    state: &asm::parser::State,
    rule_ref: asm::RuleRef,
    subparser: &mut syntax::Parser)
    -> Result<Vec<asm::RuleInvocationCandidate>, ()>
{
    let rule_group = &state.asm_state.rulesets[rule_ref.ruleset_ref.index];
    let rule = &rule_group.rules[rule_ref.index];

    let mut args = Vec::new();
    
    if DEBUG
    {
        println!("");
        println!(
            "> try match rule `{}`",
            state.fileserver.get_excerpt(&rule.span));
        println!(
            "  parser at `{}`",
            state.fileserver.get_excerpt(&subparser.get_next_spans(100)));
    }

    for (index, part) in rule.pattern.iter().enumerate()
    {
        match part
        {
            asm::PatternPart::Exact(c) =>
            {
                if DEBUG
                {
                    println!("- try match exact {}", c);
                }
                
                if subparser.next_partial().to_ascii_lowercase() != *c
                {
                    return Err(());
                }

                if DEBUG
                {
                    println!("  matched!");
                }

                subparser.advance_partial();
            }

            asm::PatternPart::Parameter(param_index) =>
            {
                let param = &rule.parameters[*param_index];

                match param.typ
                {
                    asm::PatternParameterType::Unspecified |
                    asm::PatternParameterType::Unsigned(_) |
                    asm::PatternParameterType::Signed(_) |
                    asm::PatternParameterType::Integer(_) =>
                    {
                        if DEBUG
                        {
                            println!("- try match expr");
                        }
                        
                        if subparser.is_at_partial()
                        {
                            match subparser.maybe_expect_partial_usize()
                            {
                                Some(value) =>
                                {
                                    let expr = expr::Value::make_integer(value).make_literal();
                                    args.push(vec![asm::RuleInvocationArgument::Expression(expr)]);
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

                            if DEBUG
                            {
                                println!(
                                    "  parser {}at `{}`",
                                    if expr_using_slice { "using slice " } else { "" },
                                    state.fileserver.get_excerpt(&expr_parser.get_next_spans(100)));
                            }

                            let expr = expr::Expr::parse(&mut expr_parser)?;

                            if expr_using_slice && !expr_parser.is_over()
                            {
                                return Err(());
                            }

                            args.push(vec![asm::RuleInvocationArgument::Expression(expr)]);

                            if !expr_using_slice
                            {
                                subparser.restore(expr_parser.save());
                            }

                            if DEBUG
                            {
                                println!(
                                    "  continue with parser at `{}`",
                                    state.fileserver.get_excerpt(&subparser.get_next_spans(100)));
                                    
                                println!("  matched!");
                            }
                        }
                    }

                    asm::PatternParameterType::Ruleset(rule_group_ref)=>
                    {
                        if DEBUG
                        {
                            println!("- try match subrule {:?}", rule_group_ref);
                        }

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

                        if DEBUG
                        {
                            println!("  matched!");
                        }

                        subparser.restore(subcandidates[0].1.save());
                        args.push(subcandidates
                            .into_iter()
                            .map(|s| asm::RuleInvocationArgument::NestedRuleset(s.0))
                            .collect());
                    }
                }
            }
        }
    }

    if subparser.is_at_partial()
    {
        return Err(());
    }

    // Create a candidate for each combination of arguments,
    // flattening all the alternatives
    let mut num_combinations = 1;
    for arg in &args
    {
        num_combinations *= arg.len();
    }

    let mut candidates = Vec::new();

    for combination_index in 0..num_combinations
    {
        let mut candidate = asm::RuleInvocationCandidate
        {
            rule_ref,
            specificity: 0,
            args: Vec::new(),
        };

        let mut permutation = combination_index;
        for arg in &args
        {
            let choice = permutation % arg.len();
            candidate.args.push(arg[choice].clone());

            permutation /= arg.len();
        }

        candidates.push(candidate);
    }

    Ok(candidates)
}