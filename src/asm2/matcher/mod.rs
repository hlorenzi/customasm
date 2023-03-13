use crate::*;


type WorkingMatches<'tokens> = Vec<WorkingMatch<'tokens>>;


type WorkingMatch<'tokens> =
    (InstructionMatch, syntax::TokenWalker<'tokens>);


pub type InstructionMatches = Vec<InstructionMatch>;


#[derive(Clone, Debug)]
pub struct InstructionMatch
{
    pub ruledef_ref: asm2::ItemRef<asm2::Ruledef>,
    pub rule_ref: asm2::ItemRef<asm2::Rule>,
    pub args: Vec<InstructionArgument>,
}


#[derive(Clone, Debug)]
pub struct InstructionArgument
{
    pub kind: InstructionArgumentKind,
    pub tokens: Vec<syntax::Token>,
}


#[derive(Clone, Debug)]
pub enum InstructionArgumentKind
{
    Expr(expr::Expr),
    NestedRuledef(InstructionMatch),
}


/// Runs the instruction-matching algorithm on all reachable
/// AST instruction-nodes, storing the found matches in the
/// AST nodes themselves.
pub fn match_all(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    defs: &asm2::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::Instruction(ast_instr) = any_node
        {
            let matches = match_instr(
                defs,
                ast_instr)?;

            
            if matches.len() == 0
            {
                report.error_span(
                    "no match found for instruction",
                    &ast_instr.span);
            }

            println!("instr matches: {:#?}", matches);
            ast_instr.matches = matches;
        }
    }

    Ok(())
}


/// Runs the instruction-matching algorithm on the given
/// AST instruction-node, and returns the matches.
pub fn match_instr(
    defs: &asm2::ItemDefs,
    ast_instr: &asm2::AstInstruction)
    -> Result<InstructionMatches, ()>
{
    let mut matches = WorkingMatches::new();

    for i in 0..defs.ruledefs.defs.len()
    {
        let ruledef_ref = asm2::ItemRef::<asm2::Ruledef>::new(i);

        let mut walker = syntax::TokenWalker::new(&ast_instr.tokens);

        let ruledef_matches = match_with_ruledef(
            defs,
            ruledef_ref,
            &mut walker,
            true)?;

        matches.extend(ruledef_matches);
    }

    Ok(matches.into_iter().map(|m| m.0).collect())
}


fn match_with_ruledef<'tokens>(
    defs: &asm2::ItemDefs,
    ruledef_ref: asm2::ItemRef<asm2::Ruledef>,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool)
    -> Result<WorkingMatches<'tokens>, ()>
{
    let mut matches = WorkingMatches::new();

    let ruledef = defs.ruledefs.get(ruledef_ref);

    for rule_index in 0..ruledef.rules.len()
    {
        let rule_ref = asm2::ItemRef::<asm2::Rule>::new(rule_index);
        let rule = &ruledef.rules[rule_index];

        let rule_matches = match_with_rule(
            defs,
            rule,
            &mut walker.clone(),
            needs_consume_all_tokens,
            0,
            &mut InstructionMatch {
                ruledef_ref,
                rule_ref,
                args: Vec::new(),
            })?;
            
        matches.extend(rule_matches);
    }

    Ok(matches)
}


fn match_with_rule<'tokens>(
    defs: &asm2::ItemDefs,
    rule: &asm2::Rule,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> Result<WorkingMatches<'tokens>, ()>
{
    for part_index in at_pattern_part..rule.pattern.len()
    {
        let part = &rule.pattern[part_index];

        match part
        {
            asm2::RulePatternPart::Exact(c) =>
            {
                if walker.next_is_whitespace() &&
                    !walker.is_whitespace_acknowledged()
                {
                    return Ok(vec![]);
                }
                
                if walker.next_partial().to_ascii_lowercase() != *c
                {
                    return Ok(vec![]);
                }
                
                walker.advance_partial();
            }

            asm2::RulePatternPart::Whitespace =>
            {
                if let None = walker.maybe_expect_whitespace()
                {
                    return Ok(vec![]);
                }
            }

            asm2::RulePatternPart::ParameterIndex(param_index) =>
            {
                let param = &rule.parameters[*param_index];

                match param.typ
                {
                    asm2::RuleParameterType::Unspecified |
                    asm2::RuleParameterType::Unsigned(_) |
                    asm2::RuleParameterType::Signed(_) |
                    asm2::RuleParameterType::Integer(_) =>
                    {
                        return match_with_expr(
                            defs,
                            rule,
                            walker,
                            needs_consume_all_tokens,
                            part_index,
                            match_so_far);
                    }

                    asm2::RuleParameterType::RuledefRef(ruledef_ref) =>
                    {
                        return match_with_nested_ruledef(
                            defs,
                            ruledef_ref,
                            rule,
                            walker,
                            needs_consume_all_tokens,
                            part_index,
                            match_so_far);
                    }
                }
            }
        }
    }

    if !walker.is_over() && needs_consume_all_tokens
    {
        Ok(vec![])
    }
    else
    {
        Ok(vec![(match_so_far.clone(), walker.clone())])
    }
}


fn match_with_expr<'tokens>(
    defs: &asm2::ItemDefs,
    rule: &asm2::Rule,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> Result<WorkingMatches<'tokens>, ()>
{
    if walker.is_at_partial()
    {
        match walker.maybe_expect_partial_usize()
        {
            None => Ok(vec![]),
            Some(value) =>
            {
                let expr = expr::Value::make_integer(value).make_literal();

                match_so_far.args.push(InstructionArgument {
                    kind: InstructionArgumentKind::Expr(expr),
                    tokens: Vec::new(),
                });

                Ok(vec![(match_so_far.clone(), walker.clone())])
            }
        }
    }
    else
    {
        let token_start = walker.get_current_token_index();

        let maybe_expr = parse_with_lookahead(
            &rule.pattern,
            at_pattern_part,
            walker,
            |walker| expr::parse_optional(walker));

        let token_end = walker.get_current_token_index();

        let expr = {
            match maybe_expr
            {
                Some(expr) => expr,
                None => return Ok(vec![]),
            }
        };

        match_so_far.args.push(InstructionArgument {
            kind: InstructionArgumentKind::Expr(expr),
            tokens: walker.get_cloned_tokens_by_index(
                token_start,
                token_end),
        });

        match_with_rule(
            defs,
            rule,
            walker,
            needs_consume_all_tokens,
            at_pattern_part + 1,
            match_so_far)
    }
}


fn match_with_nested_ruledef<'tokens>(
    defs: &asm2::ItemDefs,
    nested_ruledef_ref: asm2::ItemRef<asm2::Ruledef>,
    rule: &asm2::Rule,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> Result<WorkingMatches<'tokens>, ()>
{
    let token_start = walker.get_current_token_index();

    let nested_matches = parse_with_lookahead(
        &rule.pattern,
        at_pattern_part,
        walker,
        |walker| match_with_ruledef(
            defs,
            nested_ruledef_ref,
            walker,
            false))?;

    
    let mut matches = WorkingMatches::new();

    for nested_match in nested_matches
    {
        let mut walker = walker.clone();
        walker.copy_state_from(&nested_match.1);
        

        let mut match_so_far = match_so_far.clone();

        match_so_far.args.push(InstructionArgument {
            kind: InstructionArgumentKind::NestedRuledef(nested_match.0),
            tokens: walker.get_cloned_tokens_by_index(
                token_start,
                walker.get_current_token_index()),
        });

        
        // Continue matching the current rule
        let resumed_matches = match_with_rule(
            defs,
            rule,
            &mut walker,
            needs_consume_all_tokens,
            at_pattern_part + 1,
            &mut match_so_far)?;
            
        matches.extend(resumed_matches);
    }

    Ok(matches)
}


/// Cuts off the TokenWalker at the lookahead character
/// (if applicable), then runs the given parsing function with it.
/// 
/// If it is cut off, the TokenWalker state is then copied back
/// into the original, so that it is transparent to the caller.
/// 
/// This is intended to parse things like `{x} + {y}`, where the
/// `+` token is ambiguous: it can be considered part of the
/// `{x}` expression as the addition operator, or it can be an
/// "exact" pattern-part between two expressions.
/// 
/// This function forces the behavior to be the latter case, always
/// stopping the expression-parser before the `+` token, so
/// that the rest of the instruction-matching process can consume
/// it as an "exact" pattern-part.
/// 
/// By cutting off the TokenWalker, the expression-parser won't
/// be able to see the `+` token, nor anything that comes after it.
/// 
/// This behavior can be overriden by the user by using a
/// parenthesized expression, such as `(1 + 2) + 3`, where `(1 + 2)`
/// and `3` will be parsed as `{x}` and `{y}`, respectively.
/// 
/// In cases where there's no lookahead character, the TokenWalker
/// isn't cut off, and the expression-parser is allowed to
/// consume as much as it can.
fn parse_with_lookahead<'tokens, F, T>(
    pattern: &asm2::RulePattern,
    at_pattern_part: usize,
    walker: &mut syntax::TokenWalker<'tokens>,
    parse_fn: F)
    -> T
    where F: FnOnce(&mut syntax::TokenWalker<'tokens>) -> T
{
    let maybe_lookahead = find_lookahead_character(
        pattern,
        at_pattern_part);

    if let Some(lookahead) = maybe_lookahead
    {
        let maybe_cutoff_walker = walker
            .cutoff_at_char_while_respecting_parens(lookahead);
    
        if let Some(mut cutoff_walker) = maybe_cutoff_walker
        {
            let result = parse_fn(&mut cutoff_walker);
            walker.copy_state_from(&cutoff_walker);
            return result;
        }
    }

    parse_fn(walker)
}


/// Finds the next "exact" pattern-part, skipping over
/// whitespace pattern-parts, and returns its `char`.
/// 
/// If the next applicable pattern-part is a parameter,
/// it returns `None`.
fn find_lookahead_character(
    pattern: &[asm2::RulePatternPart],
    at_pattern_part: usize)
    -> Option<char>
{
    let mut i = at_pattern_part + 1;

    while i < pattern.len()
    {
        if let asm2::RulePatternPart::Whitespace = pattern[i]
        {
            i += 1;
            continue;
        }

        if let asm2::RulePatternPart::Exact(c) = pattern[i]
        {
            return Some(c);
        }

        return None;
    }

    None
}