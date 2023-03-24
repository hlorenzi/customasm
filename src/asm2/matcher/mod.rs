use crate::*;


type WorkingMatches<'tokens> = Vec<WorkingMatch<'tokens>>;


type WorkingMatch<'tokens> =
    (InstructionMatch, syntax::TokenWalker<'tokens>);


pub type InstructionMatches = Vec<InstructionMatch>;


#[derive(Clone, Debug)]
pub struct InstructionMatch
{
    pub ruledef_ref: util::ItemRef<asm2::Ruledef>,
    pub rule_ref: util::ItemRef<asm2::Rule>,
    pub args: Vec<InstructionArgument>,
    pub exact_part_count: usize,
    pub encoding_size: usize,
    pub encoding: InstructionMatchResolution,
}


#[derive(Clone, Debug)]
pub enum InstructionMatchResolution
{
    Unresolved,
    FailedConstraint(diagn::Message),
    Resolved(util::BigInt),
}


impl InstructionMatchResolution
{
    pub fn is_resolved(&self) -> bool
    {
        match self
        {
            InstructionMatchResolution::Resolved(_) => true,
            InstructionMatchResolution::FailedConstraint(_) => false,
            InstructionMatchResolution::Unresolved => false,
        }
    }


    pub fn is_resolved_or_failed(&self) -> bool
    {
        match self
        {
            InstructionMatchResolution::Resolved(_) => true,
            InstructionMatchResolution::FailedConstraint(_) => true,
            InstructionMatchResolution::Unresolved => false,
        }
    }


    pub fn unwrap_resolved(&self) -> &util::BigInt
    {
        match self
        {
            InstructionMatchResolution::Resolved(ref bigint) => bigint,
            InstructionMatchResolution::FailedConstraint(_) => panic!(),
            InstructionMatchResolution::Unresolved => panic!(),
        }
    }
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
    Nested(InstructionMatch),
}


/// Runs the instruction-matching algorithm on all reachable
/// AST instruction-nodes, storing the found matches in the
/// AST nodes themselves.
pub fn match_all(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        if let asm2::AstAny::Instruction(ast_instr) = any_node
        {
            let matches = match_instr(
                defs,
                &ast_instr.tokens);

            
            if matches.len() == 0
            {
                report.error_span(
                    "no match found for instruction",
                    &ast_instr.span);

                continue;
            }


            let instr = defs.instructions.get_mut(
                ast_instr.item_ref.unwrap());
            
            instr.matches = matches;

            // Statically calculate the encoding size
            // with a pessimistic guess
            let largest_encoding = instr.matches
                .iter()
                .max_by_key(|m| m.encoding_size)
                .unwrap();

            instr.encoding = util::BigInt::new(
                0,
                Some(largest_encoding.encoding_size));

            println!("static size for {} = {:?}",
                ast_instr.tokens.iter().map(|t| t.text()).collect::<Vec<_>>().join(""),
                instr.encoding.size.unwrap());
        }
    }

    report.stop_at_errors()
}


/// Runs the instruction-matching algorithm on the given
/// Token slice, and returns the matches.
pub fn match_instr(
    defs: &asm2::ItemDefs,
    tokens: &[syntax::Token])
    -> InstructionMatches
{
    let mut working_matches = WorkingMatches::new();

    for i in 0..defs.ruledefs.defs.len()
    {
        let ruledef_ref = util::ItemRef::<asm2::Ruledef>::new(i);

        let mut walker = syntax::TokenWalker::new(tokens);

        let ruledef_matches = match_with_ruledef(
            defs,
            ruledef_ref,
            &mut walker,
            true);

        working_matches.extend(ruledef_matches);
    }

    if working_matches.len() == 0
    {
        return vec![];
    }


    let mut matches = working_matches
        .into_iter()
        .map(|m| m.0)
        .collect::<Vec<_>>();


    // Calculate recursive "exact" pattern-part count for
    // each match
    for mtch in &mut matches
    {
        mtch.exact_part_count = get_recursive_exact_part_count(
            defs,
            mtch);
    }


    // Only keep matches with the maximum count of
    // "exact" pattern-parts
    let max_exact_count = matches
        .iter()
        .max_by_key(|m| m.exact_part_count)
        .unwrap()
        .exact_part_count;

    matches.retain(|c| c.exact_part_count == max_exact_count);


    // Statically calculate the encoding size of each match,
    // whenever possible.
    for mtch in &mut matches
    {
        mtch.encoding_size = get_match_static_size(defs, &mtch)
            .unwrap_or(0);
    }


    matches
}


fn get_match_static_size(
    defs: &asm2::ItemDefs,
    mtch: &asm2::InstructionMatch)
    -> Option<usize>
{
    let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
    let rule = &ruledef.get_rule(mtch.rule_ref);

    let mut info = expr::StaticSizeInfo::new();
    
    for i in 0..rule.parameters.len()
    {
        let param = &rule.parameters[i];
        let arg = &mtch.args[i];

        match param.typ
        {
            asm2::RuleParameterType::Unspecified => {}

            asm2::RuleParameterType::Integer(size) |
            asm2::RuleParameterType::Unsigned(size) |
            asm2::RuleParameterType::Signed(size) =>
            {
                info.locals.insert(param.name.clone(), size);
            }

            asm2::RuleParameterType::RuledefRef(_) =>
            {
                if let asm2::InstructionArgumentKind::Nested(ref nested_match) = arg.kind
                {
                    let maybe_nested_size = get_match_static_size(
                        defs,
                        nested_match);
                    
                    if let Some(nested_size) = maybe_nested_size
                    {
                        info.locals.insert(param.name.clone(), nested_size);
                    }
                }
            }
        }
    }

    rule.expr.get_static_size(&info)
}


fn match_with_ruledef<'tokens>(
    defs: &asm2::ItemDefs,
    ruledef_ref: util::ItemRef<asm2::Ruledef>,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool)
    -> WorkingMatches<'tokens>
{
    let mut matches = WorkingMatches::new();

    let ruledef = defs.ruledefs.get(ruledef_ref);

    for rule_ref in ruledef.iter_rule_refs()
    {
        let rule = &ruledef.get_rule(rule_ref);

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
                exact_part_count: 0,
                encoding_size: 0,
                encoding: InstructionMatchResolution::Unresolved,
            });
            
        matches.extend(rule_matches);
    }

    matches
}


fn match_with_rule<'tokens>(
    defs: &asm2::ItemDefs,
    rule: &asm2::Rule,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> WorkingMatches<'tokens>
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
                    return vec![];
                }
                
                if walker.next_partial().to_ascii_lowercase() != *c
                {
                    return vec![];
                }
                
                walker.advance_partial();
            }

            asm2::RulePatternPart::Whitespace =>
            {
                if let None = walker.maybe_expect_whitespace()
                {
                    return vec![];
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
        vec![]
    }
    else
    {
        vec![(match_so_far.clone(), walker.clone())]
    }
}


fn match_with_expr<'tokens>(
    defs: &asm2::ItemDefs,
    rule: &asm2::Rule,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> WorkingMatches<'tokens>
{
    if walker.is_at_partial()
    {
        match walker.maybe_expect_partial_usize()
        {
            None => vec![],
            Some(value) =>
            {
                let expr = expr::Value::make_integer(value)
                    .make_literal();

                match_so_far.args.push(InstructionArgument {
                    kind: InstructionArgumentKind::Expr(expr),
                    tokens: Vec::new(),
                });

                vec![(match_so_far.clone(), walker.clone())]
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
                None => return vec![],
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
    nested_ruledef_ref: util::ItemRef<asm2::Ruledef>,
    rule: &asm2::Rule,
    walker: &mut syntax::TokenWalker<'tokens>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> WorkingMatches<'tokens>
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
            false));

    
    let mut matches = WorkingMatches::new();

    for nested_match in nested_matches
    {
        let mut walker = walker.clone();
        walker.copy_state_from(&nested_match.1);
        

        let mut match_so_far = match_so_far.clone();

        match_so_far.args.push(InstructionArgument {
            kind: InstructionArgumentKind::Nested(nested_match.0),
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
            &mut match_so_far);
            
        matches.extend(resumed_matches);
    }

    matches
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


fn get_recursive_exact_part_count(
    defs: &asm2::ItemDefs,
    instr_match: &InstructionMatch)
    -> usize
{
    let mut count = 0;

    for arg in &instr_match.args
    {
        if let InstructionArgumentKind::Nested(ref nested_match) = arg.kind
        {
            count += get_recursive_exact_part_count(
                defs,
                nested_match);
        }
    }

    let ruledef = defs.ruledefs.get(instr_match.ruledef_ref);
    let rule = &ruledef.rules[instr_match.rule_ref.0];

    count + rule.exact_part_count
}