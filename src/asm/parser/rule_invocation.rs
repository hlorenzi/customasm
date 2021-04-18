use crate::*;


static DEBUG: bool = false;


pub fn parse_rule_invocation(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let mut subparser = state.parser.slice_until_linebreak();
    subparser.suppress_reports();

    let ctx = state.asm_state.get_ctx(&state);

    if let Ok(invocation) = match_rule_invocation(
        &mut state.asm_state,
        subparser,
        ctx,
        state.fileserver,
        state.report.clone())
    {
        let bankdata = state.asm_state.get_bankdata(state.asm_state.cur_bank);
        bankdata.check_writable(&state.asm_state, state.report.clone(), &invocation.span)?;
        
        let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
        bankdata.push_invocation(invocation);
    }

    state.parser.expect_linebreak()?;
    Ok(())
}


pub fn match_rule_invocation(
    asm_state: &asm::State,
    subparser: syntax::Parser,
    ctx: asm::Context,
    fileserver: &dyn util::FileServer,
    report: diagn::RcReport)
    -> Result<asm::Invocation, ()>
{
    if DEBUG
    {
        println!("");
        println!(
            "=== parse rule invocation `{}` ===",
            fileserver.get_excerpt(&subparser.get_full_span()));
    }

    let mut candidates = match_active_rulesets(asm_state, &subparser, fileserver, report.clone())?;
    if candidates.len() != 0
    {
        // Calculate specificity scores
        for candidate in &mut candidates
        {
            candidate.specificity = candidate.calculate_specificity_score(&asm_state);
        }

        // Sort candidates by specificity score
        candidates.sort_by(|a, b| b.specificity.cmp(&a.specificity));
        
        if DEBUG
        {
            println!("");
            println!("final candidates:");
            for candidate in &candidates
            {
                let rule_group = &asm_state.rulesets[candidate.rule_ref.ruleset_ref.index];
                let rule = &rule_group.rules[candidate.rule_ref.index];

                println!(
                    "  `{}`",
                    fileserver.get_excerpt(&rule.span));
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
            ctx,
            size_guess: 0,
            span: subparser.get_full_span(),
            kind: asm::InvocationKind::Rule(asm::RuleInvocation
            {
                candidates,
            })
        };
        
        let resolved = asm_state.resolve_rule_invocation(
            report.clone(),
            &invocation,
            fileserver,
            false,
            &mut expr::EvalContext::new());

        //println!("{} = {:?}", fileserver.get_excerpt(&invocation.span), &resolved);

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

        //println!("{} = {}", fileserver.get_excerpt(&invocation.span), invocation.size_guess);

        return Ok(invocation);
    }

    Err(())
}


pub fn match_active_rulesets(
    asm_state: &asm::State,
    subparser: &syntax::Parser,
    fileserver: &dyn util::FileServer,
    report: diagn::RcReport)
    -> Result<Vec<asm::RuleInvocationCandidate>, ()>
{
    let mut candidates = Vec::new();

    for ruleset_ref in &asm_state.active_rulesets
    {
        if let Ok(subcandidates) = match_ruleset(asm_state, *ruleset_ref, &subparser, true, fileserver, report.clone())
        {
            for candidate in subcandidates
            {
                candidates.push(candidate.0);
            }
        }
    }

    if candidates.len() == 0
    {
        report.error_span("no match for instruction found", &subparser.get_full_span());
    }

    //println!(
    //    "rule candidates for `{}`:\n{:#?}",
    //    fileserver.get_excerpt(&subparser.get_full_span()),
    //    candidates);

    Ok(candidates)
}


pub fn match_ruleset<'a>(
    asm_state: &asm::State,
    ruleset_ref: asm::RulesetRef,
    subparser: &syntax::Parser<'a>,
    must_consume_all_tokens: bool,
    fileserver: &dyn util::FileServer,
    report: diagn::RcReport)
    -> Result<Vec<(asm::RuleInvocationCandidate, syntax::Parser<'a>)>, ()>
{
    let rule_group = &asm_state.rulesets[ruleset_ref.index];

    let mut candidates = Vec::new();

    for index in 0..rule_group.rules.len()
    {
        let rule_ref = asm::RuleRef
        {
            ruleset_ref,
            index,
        };

        if let Ok(subcandidates) = match_rule(asm_state, rule_ref, subparser, fileserver, report.clone())
        {
            //println!(
            //    "finish pattern with parser at `{}`",
            //    fileserver.get_excerpt(&subparser_clone.get_next_spans(10)));
        
            for subcandidate in subcandidates
            {
                if !must_consume_all_tokens || subcandidate.1.is_over()
                {
                    candidates.push(subcandidate);
                }
            }
        }
    }

    Ok(candidates)
}


#[derive(Clone)]
struct ParsingBranch<'a>
{
    args: Vec<asm::RuleInvocationArgument>,
    token_args: Vec<Option<Vec<syntax::Token>>>,
    parser: syntax::Parser<'a>,
    dead: bool,
}


pub fn match_rule<'a>(
    asm_state: &asm::State,
    rule_ref: asm::RuleRef,
    subparser: &syntax::Parser<'a>,
    fileserver: &dyn util::FileServer,
    report: diagn::RcReport)
    -> Result<Vec<(asm::RuleInvocationCandidate, syntax::Parser<'a>)>, ()>
{
    let rule_group = &asm_state.rulesets[rule_ref.ruleset_ref.index];
    let rule = &rule_group.rules[rule_ref.index];

    let mut parsing_branches = Vec::new();
    parsing_branches.push(ParsingBranch
    {
        args: Vec::new(),
        token_args: Vec::new(),
        parser: subparser.clone(),
        dead: false,
    });
    
    if DEBUG
    {
        println!("");
        println!(
            "> try match rule `{}`",
            fileserver.get_excerpt(&rule.span));
        println!(
            "  parser at `{}`",
            fileserver.get_excerpt(&subparser.get_next_spans(100)));
    }

    for (index, part) in rule.pattern.iter().enumerate()
    {
        parsing_branches.retain(|b| !b.dead);
        if parsing_branches.len() == 0
        {
            break;
        }

        let mut new_branches = Vec::new();
        
        for (branch_index, branch) in parsing_branches.iter_mut().enumerate()
        {
            match part
            {
                asm::PatternPart::Exact(c) =>
                {
                    if DEBUG
                    {
                        println!("- branch {}, try match exact `{}`", branch_index, c);
                    }
                    
                    if branch.parser.next_partial().to_ascii_lowercase() != *c
                    {
                        branch.dead = true;
                    }
                    else
                    {
                        branch.parser.advance_partial();

                        if DEBUG
                        {
                            println!("  branch {}, exact matched! parser at `{}`",
                                branch_index,
                                fileserver.get_excerpt(&branch.parser.get_next_spans(100)));
                        }
                    }
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
                                println!("- branch {}, try match expr", branch_index);
                            }
                            
                            if branch.parser.is_at_partial()
                            {
                                match branch.parser.maybe_expect_partial_usize()
                                {
                                    Some(value) =>
                                    {
                                        let expr = expr::Value::make_integer(value).make_literal();
                                        branch.args.push(asm::RuleInvocationArgument::Expression(expr));
                                        branch.token_args.push(None);
                                    }
                                    None =>
                                    {
                                        branch.dead = true;
                                    }
                                }
                            }
                            else
                            {
                                let mut expr_parser = branch.parser.clone();
                                let mut expr_using_slice = false;

                                let next_part = rule.pattern.get(index + 1);

                                if let Some(asm::PatternPart::Exact(next_part_char)) = next_part
                                {
                                    if let Some(slice_parser) = branch.parser.slice_until_char_or_nesting(*next_part_char)
                                    {
                                        expr_parser = slice_parser;
                                        expr_using_slice = true;
                                    }
                                }

                                if DEBUG
                                {
                                    println!(
                                        "  branch {}, parser {}at `{}`",
                                        branch_index,
                                        if expr_using_slice { "using slice " } else { "" },
                                        fileserver.get_excerpt(&expr_parser.get_next_spans(100)));
                                }

                                let expr = expr::Expr::parse(&mut expr_parser)?;

                                if expr_using_slice && !expr_parser.is_over()
                                {
                                    branch.dead = true;
                                }
                                else
                                {
                                    branch.args.push(asm::RuleInvocationArgument::Expression(expr));
                                    branch.token_args.push(None);

                                    if !expr_using_slice
                                    {
                                        branch.parser.restore(expr_parser.save());
                                    }

                                    if DEBUG
                                    {
                                        println!("  branch {}, expr matched! parser at `{}`",
                                            branch_index,
                                            fileserver.get_excerpt(&branch.parser.get_next_spans(100)));
                                    }
                                }
                            }
                        }

                        asm::PatternParameterType::Ruleset(rule_group_ref) =>
                        {
                            if DEBUG
                            {
                                println!("- branch {}, try match subrule {:?}", branch_index, rule_group_ref);
                            }

                            let token_start = branch.parser.get_current_token_index();

                            let subcandidates = match_ruleset(
                                asm_state,
                                rule_group_ref,
                                &mut branch.parser,
                                false,
                                fileserver,
                                report.clone())?;

                            if subcandidates.len() != 0
                            {
                                if DEBUG
                                {
                                    println!("  branch {}, {} subrules matched!", branch_index, subcandidates.len());
                                }

                                for subcandidate in subcandidates.into_iter()
                                {
                                    let token_end = subcandidate.1.get_current_token_index();

                                    let mut args_clone = branch.args.clone();
                                    args_clone.push(asm::RuleInvocationArgument::NestedRuleset(subcandidate.0));

                                    let mut token_args_clone = branch.token_args.clone();
                                    token_args_clone.push(Some(branch.parser.get_cloned_tokens_by_index(token_start, token_end)));

                                    new_branches.push(ParsingBranch
                                    {
                                        args: args_clone,
                                        token_args: token_args_clone,
                                        parser: subcandidate.1,
                                        dead: false,
                                    });
                                }
                            }

                            branch.dead = true;
                        }
                    }
                }
            }
        }

        for new_branch in new_branches.into_iter()
        {
            parsing_branches.push(new_branch);
        }
    }

    if DEBUG
    {
        println!("  end try match rule");
    }
    
    let mut candidates = Vec::new();

    for (branch_index, branch) in parsing_branches.into_iter().enumerate()
    {
        if DEBUG
        {
            println!("= branch {}{}, candidate parser at `{}`",
                branch_index,
                if branch.dead { " (dead)" } else { "" },
                fileserver.get_excerpt(&branch.parser.get_next_spans(100)));
        }

        if branch.dead
        {
            continue;
        }

        let candidate = asm::RuleInvocationCandidate
        {
            rule_ref,
            specificity: 0,
            args: branch.args,
            token_args: branch.token_args,
        };

        candidates.push((candidate, branch.parser));
    }

    Ok(candidates)
}