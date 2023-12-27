use crate::*;


type WorkingMatches<'src> = Vec<WorkingMatch<'src>>;


type WorkingMatch<'src> =
    (InstructionMatch, syntax::Walker<'src>);


pub type InstructionMatches = Vec<InstructionMatch>;


#[derive(Clone, Debug)]
pub struct InstructionMatch
{
    pub ruledef_ref: util::ItemRef<asm::Ruledef>,
    pub rule_ref: util::ItemRef<asm::Rule>,
    pub args: Vec<InstructionArgument>,
    pub exact_part_count: usize,
    pub encoding_statically_known: bool,
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
    pub span: diagn::Span,
    pub excerpt: String,
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
    opts: &asm::AssemblyOptions,
    ast: &asm::AstTopLevel,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    let mut symbol_ctx = &util::SymbolContext::new_global();


    for any_node in &ast.nodes
    {
        if let asm::AstAny::Instruction(ast_instr) = any_node
        {
            let mut matches = match_instr(
                opts,
                defs,
                ast_instr.span,
                &ast_instr.src);

            
            if let Err(()) = error_on_no_matches(
                report,
                ast_instr.span,
                &matches)
            {
                continue;
            }


            // Statically calculate information for each match,
            // whenever possible.
            for mtch in &mut matches
            {
                mtch.encoding_statically_known = get_match_statically_known(
                    decls,
                    defs,
                    symbol_ctx,
                    &mtch);
                
                mtch.encoding_size = get_match_static_size(defs, &mtch)
                    .unwrap_or(0);
            }


            let instr = defs.instructions.get_mut(
                ast_instr.item_ref.unwrap());
            
            instr.matches = matches;

            instr.encoding_statically_known = instr.matches.iter()
                .all(|m| m.encoding_statically_known);

            // Statically calculate the encoding size
            // with a pessimistic guess
            let largest_encoding = instr.matches
                .iter()
                .max_by_key(|m| m.encoding_size)
                .unwrap();

            instr.encoding = util::BigInt::new(
                0,
                Some(largest_encoding.encoding_size));

            if opts.debug_iterations
            {
                println!(" size: {} = {:?}{}",
                    ast_instr.src,
                    instr.encoding.size.unwrap(),
                    if instr.encoding_statically_known { " [static]" } else { "" });
            }
        }

        else if let asm::AstAny::Symbol(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();
            symbol_ctx = &decls.symbols.get(item_ref).ctx;
        }
    }

    report.stop_at_errors()
}


pub fn error_on_no_matches(
    report: &mut diagn::Report,
    span: diagn::Span,
    matches: &InstructionMatches)
    -> Result<(), ()>
{
    if matches.len() == 0
    {
        report.error_span(
            "no match found for instruction",
            span);

        Err(())
    }
    else
    {
        Ok(())
    }
}


fn get_match_statically_known(
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    symbol_ctx: &util::SymbolContext,
    mtch: &asm::InstructionMatch)
    -> bool
{
    let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
    let rule = &ruledef.get_rule(mtch.rule_ref);

    let query_variable = |query: &expr::StaticallyKnownVariableQuery|
    {
        match decls.symbols.try_get_by_name(
            symbol_ctx,
            query.hierarchy_level,
            query.hierarchy)
        {
            None => false,
            Some(symbol_ref) =>
            {
                let symbol = defs.symbols.get(symbol_ref);
                symbol.value_statically_known
            }
        }
    };

    let mut provider = expr::StaticallyKnownProvider::new();
    provider.query_variable = &query_variable;
    provider.query_function = &asm::resolver::get_statically_known_builtin_fn;

    for i in 0..rule.parameters.len()
    {
        let param = &rule.parameters[i];
        let arg = &mtch.args[i];

        match param.typ
        {
            asm::RuleParameterType::Unspecified |
            asm::RuleParameterType::Integer(_) |
            asm::RuleParameterType::Unsigned(_) |
            asm::RuleParameterType::Signed(_) =>
            {
                if let InstructionArgumentKind::Expr(ref arg_expr) = arg.kind
                {
                    if arg_expr.is_value_statically_known(&provider)
                    {
                        provider.locals.insert(
                            param.name.clone(),
                            expr::StaticallyKnownLocal {
                                value_known: true,
                                ..expr::StaticallyKnownLocal::new()
                            });
                    }
                }
            }

            asm::RuleParameterType::RuledefRef(_) =>
            {
                if let asm::InstructionArgumentKind::Nested(ref nested_match) = arg.kind
                {
                    if get_match_statically_known(
                        decls,
                        defs,
                        symbol_ctx,
                        nested_match)
                    {
                        provider.locals.insert(
                            param.name.clone(),
                            expr::StaticallyKnownLocal {
                                value_known: true,
                                ..expr::StaticallyKnownLocal::new()
                            });
                    }
                }
            }
        }
    }

    rule.expr.is_value_statically_known(&provider)
}


fn get_match_static_size(
    defs: &asm::ItemDefs,
    mtch: &asm::InstructionMatch)
    -> Option<usize>
{
    let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
    let rule = &ruledef.get_rule(mtch.rule_ref);

    let mut info = expr::StaticallyKnownProvider::new();

    for i in 0..rule.parameters.len()
    {
        let param = &rule.parameters[i];
        let arg = &mtch.args[i];

        match param.typ
        {
            asm::RuleParameterType::Unspecified => {}

            asm::RuleParameterType::Integer(size) |
            asm::RuleParameterType::Unsigned(size) |
            asm::RuleParameterType::Signed(size) =>
            {
                info.locals.insert(
                    param.name.clone(),
                    expr::StaticallyKnownLocal {
                        size: Some(size),
                        ..expr::StaticallyKnownLocal::new()
                    });
            }

            asm::RuleParameterType::RuledefRef(_) =>
            {
                if let asm::InstructionArgumentKind::Nested(ref nested_match) = arg.kind
                {
                    let maybe_nested_size = get_match_static_size(
                        defs,
                        nested_match);
                    
                    if let Some(nested_size) = maybe_nested_size
                    {
                        info.locals.insert(
                            param.name.clone(),
                            expr::StaticallyKnownLocal {
                                size: Some(nested_size),
                                ..expr::StaticallyKnownLocal::new()
                            });
                    }
                }
            }
        }
    }

    rule.expr.get_static_size(&info)
}


/// Runs the instruction-matching algorithm on the given
/// string, and returns the matches.
pub fn match_instr(
    opts: &asm::AssemblyOptions,
    defs: &asm::ItemDefs,
    span: diagn::Span,
    src: &str)
    -> InstructionMatches
{
    let mut working_matches = WorkingMatches::new();
    let mut walker = syntax::Walker::new(
        src,
        span.file_handle,
        span.location().unwrap().0 as usize);

    if opts.optimize_instruction_matching
    {
        let ruledef_matches = match_with_ruledef_map(
            defs,
            walker);

        working_matches.extend(ruledef_matches);
    }

    else
    {
        for i in 0..defs.ruledefs.defs.len()
        {
            let ruledef_ref = util::ItemRef::<asm::Ruledef>::new(i);
            let ruledef = defs.ruledefs.get(ruledef_ref);

            if ruledef.is_subruledef
            {
                continue;
            }

            let ruledef_matches = match_with_ruledef(
                defs,
                ruledef_ref,
                &mut walker,
                true);

            working_matches.extend(ruledef_matches);
        }
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


    matches
}


fn match_with_ruledef_map<'src>(
    defs: &asm::ItemDefs,
    walker: syntax::Walker<'src>)
    -> WorkingMatches<'src>
{
    let mut matches = WorkingMatches::new();

    let prefix = asm::RuledefMap::parse_prefix(&walker);
    let entries = defs.ruledef_map.query_prefixed(prefix);

    for entry in entries.iter().flat_map(|e| e.iter())
    {
        let ruledef = defs.ruledefs.get(entry.ruledef_ref);
        let rule = ruledef.get_rule(entry.rule_ref);

        let rule_matches = begin_match_with_rule(
            defs,
            entry.ruledef_ref,
            entry.rule_ref,
            rule,
            walker.clone(),
            true);
            
        matches.extend(rule_matches);
    }

    matches
}


fn match_with_ruledef<'src>(
    defs: &asm::ItemDefs,
    ruledef_ref: util::ItemRef<asm::Ruledef>,
    walker: &mut syntax::Walker<'src>,
    needs_consume_all_tokens: bool)
    -> WorkingMatches<'src>
{
    let mut matches = WorkingMatches::new();

    let ruledef = defs.ruledefs.get(ruledef_ref);

    for rule_ref in ruledef.iter_rule_refs()
    {
        let rule = &ruledef.get_rule(rule_ref);

        let rule_matches = begin_match_with_rule(
            defs,
            ruledef_ref,
            rule_ref,
            rule,
            walker.clone(),
            needs_consume_all_tokens);
            
        matches.extend(rule_matches);
    }

    matches
}


fn begin_match_with_rule<'src>(
    defs: &asm::ItemDefs,
    ruledef_ref: util::ItemRef<asm::Ruledef>,
    rule_ref: util::ItemRef<asm::Rule>,
    rule: &asm::Rule,
    mut walker: syntax::Walker<'src>,
    needs_consume_all_tokens: bool)
    -> WorkingMatches<'src>
{
    match_with_rule(
        defs,
        rule,
        &mut walker,
        needs_consume_all_tokens,
        0,
        &mut InstructionMatch {
            ruledef_ref,
            rule_ref,
            args: Vec::new(),
            exact_part_count: 0,
            encoding_statically_known: false,
            encoding_size: 0,
            encoding: InstructionMatchResolution::Unresolved,
        })
}


fn match_with_rule<'src>(
    defs: &asm::ItemDefs,
    rule: &asm::Rule,
    walker: &mut syntax::Walker<'src>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> WorkingMatches<'src>
{
    for part_index in at_pattern_part..rule.pattern.len()
    {
        let part = &rule.pattern[part_index];

        match part
        {
            asm::RulePatternPart::Exact(c) =>
            {
                if !walker.maybe_expect_char(*c)
                {
                    return vec![];
                }
            }

            asm::RulePatternPart::Whitespace =>
            {
                if walker.next_token().kind != syntax::TokenKind::Whitespace
                {
                    return vec![];
                }
            }

            asm::RulePatternPart::ParameterIndex(param_index) =>
            {
                let param = &rule.parameters[*param_index];

                match param.typ
                {
                    asm::RuleParameterType::Unspecified |
                    asm::RuleParameterType::Unsigned(_) |
                    asm::RuleParameterType::Signed(_) |
                    asm::RuleParameterType::Integer(_) =>
                    {
                        return match_with_expr(
                            defs,
                            rule,
                            walker,
                            needs_consume_all_tokens,
                            part_index,
                            match_so_far);
                    }

                    asm::RuleParameterType::RuledefRef(ruledef_ref) =>
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


fn match_with_expr<'src>(
    defs: &asm::ItemDefs,
    rule: &asm::Rule,
    walker: &mut syntax::Walker<'src>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> WorkingMatches<'src>
{
    let walker_start = walker.next_useful_index();

    let maybe_expr = parse_with_lookahead(
        &rule.pattern,
        at_pattern_part,
        walker,
        |walker| expr::parse_optional(walker));

    let walker_end = walker.get_cursor_index();
    let walker_start = std::cmp::min(walker_start, walker_end);

    let expr = {
        match maybe_expr
        {
            Some(expr) => expr,
            None => return vec![],
        }
    };

    let span = walker.get_span(
        walker_start,
        walker_end);

    let excerpt = walker.get_excerpt(
        walker_start,
        walker_end);

    match_so_far.args.push(InstructionArgument {
        kind: InstructionArgumentKind::Expr(expr),
        span,
        excerpt: excerpt.to_string(),
    });

    match_with_rule(
        defs,
        rule,
        walker,
        needs_consume_all_tokens,
        at_pattern_part + 1,
        match_so_far)
}


fn match_with_nested_ruledef<'src>(
    defs: &asm::ItemDefs,
    nested_ruledef_ref: util::ItemRef<asm::Ruledef>,
    rule: &asm::Rule,
    walker: &mut syntax::Walker<'src>,
    needs_consume_all_tokens: bool,
    at_pattern_part: usize,
    match_so_far: &mut InstructionMatch)
    -> WorkingMatches<'src>
{
    let walker_start = walker.next_useful_index();
    let walker_limit_prev = walker.get_cursor_limit();

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
        let mut walker = nested_match.1;
        walker.set_cursor_limit(walker_limit_prev);

        let walker_end = walker.get_cursor_index();
        let walker_start = std::cmp::min(walker_start, walker_end);
        
        let mut match_so_far = match_so_far.clone();

        let span = walker.get_span(
            walker_start,
            walker_end);

        let excerpt = walker.get_excerpt(
            walker_start,
            walker_end);

        match_so_far.args.push(InstructionArgument {
            kind: InstructionArgumentKind::Nested(nested_match.0),
            span,
            excerpt: excerpt.to_string(),
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
fn parse_with_lookahead<'src, F, T>(
    pattern: &asm::RulePattern,
    at_pattern_part: usize,
    walker: &mut syntax::Walker<'src>,
    parse_fn: F)
    -> T
    where F: FnOnce(&mut syntax::Walker<'src>) -> T
{
    let maybe_lookahead_char = find_lookahead_char(
        pattern,
        at_pattern_part);

    if let Some(lookahead_char) = maybe_lookahead_char
    {
        let maybe_limit =
            walker.find_lookahead_char_index(lookahead_char);
    
        if let Some(limit) = maybe_limit
        {
            let prev_limit = walker.get_cursor_limit();
            walker.set_cursor_limit(limit);
            let result = parse_fn(walker);
            walker.set_cursor_limit(prev_limit);
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
fn find_lookahead_char(
    pattern: &[asm::RulePatternPart],
    at_pattern_part: usize)
    -> Option<char>
{
    let mut i = at_pattern_part + 1;

    while i < pattern.len()
    {
        if let asm::RulePatternPart::Whitespace = pattern[i]
        {
            i += 1;
            continue;
        }

        if let asm::RulePatternPart::Exact(c) = pattern[i]
        {
            return Some(c);
        }

        return None;
    }

    None
}


fn get_recursive_exact_part_count(
    defs: &asm::ItemDefs,
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