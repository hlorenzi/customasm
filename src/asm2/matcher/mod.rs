use crate::*;


pub type InstructionMatches = Vec<InstructionMatch>;


#[derive(Debug)]
pub struct InstructionMatch
{
    pub ruledef_ref: asm2::ItemRef<asm2::Ruledef>,
    pub rule_ref: asm2::ItemRef<asm2::Rule>,
    pub args: Vec<InstructionArgument>,
}


#[derive(Debug)]
pub struct InstructionArgument
{
    pub expr: expr::Expr,
    pub tokens: Vec<syntax::Token>,
}


pub fn match_all(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstNodeAny::Instruction(ast_instr) = any_node
        {
            let matches = match_instr(
                report,
                decls,
                defs,
                ast_instr)?;

            println!("instr matches: {:#?}", matches);
            ast_instr.matches = matches;
        }
    }

    Ok(())
}


pub fn match_instr(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ast_instr: &asm2::AstInstruction)
    -> Result<InstructionMatches, ()>
{
    let mut matches = InstructionMatches::new();

    for i in 0..decls.ruledefs.decls.len()
    {
        let ruledef_ref = asm2::ItemRef::<asm2::Ruledef>::new(i);

        let walker = syntax::TokenWalker::new(&ast_instr.tokens);

        let ruledef_matches = match_with_ruledef(
            report,
            decls,
            defs,
            ruledef_ref,
            defs.ruledefs.get(ruledef_ref),
            walker)?;

        matches.extend(ruledef_matches);
    }

    Ok(matches)
}


fn match_with_ruledef(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ruledef_ref: asm2::ItemRef<asm2::Ruledef>,
    ruledef: &asm2::Ruledef,
    walker: syntax::TokenWalker)
    -> Result<InstructionMatches, ()>
{
    let mut matches = InstructionMatches::new();

    for rule_index in 0..ruledef.rules.len()
    {
        let rule_ref = asm2::ItemRef::<asm2::Rule>::new(rule_index);
        let rule = &ruledef.rules[rule_index];

        let rule_matches = match_with_rule(
            report,
            decls,
            defs,
            ruledef,
            rule,
            walker.clone(),
            0,
            InstructionMatch {
                ruledef_ref,
                rule_ref,
                args: Vec::new(),
            })?;
            
        matches.extend(rule_matches);
    }

    Ok(matches)
}


fn match_with_rule(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ruledef: &asm2::Ruledef,
    rule: &asm2::Rule,
    mut walker: syntax::TokenWalker,
    from_pattern_part: usize,
    match_so_far: InstructionMatch)
    -> Result<InstructionMatches, ()>
{
    for part_index in from_pattern_part..rule.pattern.len()
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
                else if walker.next_partial().to_ascii_lowercase() != *c
                {
                    return Ok(vec![]);
                }
                else
                {
                    walker.advance_partial();
                }
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
                            report,
                            decls,
                            defs,
                            ruledef,
                            rule,
                            walker,
                            part_index,
                            match_so_far);
                    }

                    asm2::RuleParameterType::RuledefRef(..) =>
                        todo!(),
                }
            }
        }
    }

    if !walker.is_over()
    {
        Ok(vec![])
    }
    else
    {
        Ok(vec![match_so_far])
    }
}


fn match_with_expr(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ruledef: &asm2::Ruledef,
    rule: &asm2::Rule,
    mut walker: syntax::TokenWalker,
    from_pattern_part: usize,
    mut match_so_far: InstructionMatch)
    -> Result<InstructionMatches, ()>
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
                    expr,
                    tokens: Vec::new(),
                });

                Ok(vec![match_so_far])
            }
        }
    }
    else
    {
        let next_part = get_next_non_whitespace_pattern_part(
            &rule.pattern,
            from_pattern_part);

        let token_start = walker.get_current_token_index();

        let maybe_expr = {
            if let Some(asm2::RulePatternPart::Exact(c)) = next_part
            {
                if let Some(mut new_walker) = walker.slice_until_char_or_nesting(*c)
                {
                    let maybe_expr = expr::parse_optional(&mut new_walker);
                    walker.restore(new_walker.save());
                    maybe_expr
                }
                else
                {
                    expr::parse_optional(&mut walker)
                }
            }
            else
            {
                expr::parse_optional(&mut walker)
            }
        };

        let expr = {
            match maybe_expr
            {
                Some(expr) => expr,
                None => return Ok(vec![]),
            }
        };

        let token_end = walker.get_current_token_index();

        match_so_far.args.push(InstructionArgument {
            expr,
            tokens: walker.get_cloned_tokens_by_index(
                token_start,
                token_end),
        });

        match_with_rule(
            report,
            decls,
            defs,
            ruledef,
            rule,
            walker,
            from_pattern_part + 1,
            match_so_far)
    }
}


fn get_next_non_whitespace_pattern_part(
    pattern: &[asm2::RulePatternPart],
    from_index: usize)
    -> Option<&asm2::RulePatternPart>
{
    let mut i = from_index + 1;

    while i < pattern.len()
    {
        if let asm2::RulePatternPart::Whitespace = pattern[i]
        {
            i += 1;
            continue;
        }

        return Some(&pattern[i]);
    }

    None
}