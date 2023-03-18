use crate::*;


pub fn resolve_instruction<'symbol_ctx>(
    report: &mut diagn::Report,
    ast_instr: &asm2::AstInstruction,
    decls: &'symbol_ctx asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    symbol_ctx: &'symbol_ctx util::SymbolContext)
    -> Result<asm2::ResolutionState, ()>
{
    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());


    // Skip this instruction if already resolved
    if let Some(_) = instr.chosen_encoding
    {
        return Ok(asm2::ResolutionState::Resolved);
    }


    // Try to resolve every match
    for index in 0..instr.matches.len()
    {
        let instr = defs.instructions.get(ast_instr.item_ref.unwrap());
        let mtch = &instr.matches[index];
        let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
        let rule = &ruledef.get_rule(mtch.rule_ref);


        // Skip this match if already resolved
        let maybe_prev_computed = &mtch.resolved;
        if let Some(_) = maybe_prev_computed
        {
            continue;
        }


        let value = {
            report.push_parent(
                "failed to resolve instruction",
                &ast_instr.span);
            
            report.push_parent_note(
                "while attempting the following rule candidate:",
                &rule.pattern_span);

            let maybe_value = resolve_instruction_match(
                report,
                &mtch,
                decls,
                defs,
                symbol_ctx);

            report.pop_parent();
            report.pop_parent();

            maybe_value?
        };


        if let expr::Value::Unknown = value
        {
            return Ok(asm2::ResolutionState::Unresolved);
        }


        let bigint = {
            report.push_parent(
                "failed to resolve instruction",
                &ast_instr.span);
            
            report.push_parent_note(
                "while attempting the following rule candidate:",
                &rule.pattern_span);

            let maybe_bigint = value.expect_sized_bigint(
                report,
                &rule.expr.returned_value_span());

            report.pop_parent();
            report.pop_parent();

            maybe_bigint?
        };


        let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
        instr.matches[index].resolved = Some(bigint.clone());
    }


    // Only retain the smallest encodings
    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());

    let smallest_encoding = instr.matches
        .iter()
        .map(|m| m.resolved.as_ref().unwrap())
        .min_by_key(|e| e.size.unwrap())
        .unwrap()
        .size
        .unwrap();

    let encodings = instr.matches
        .iter()
        .map(|m| m.resolved.as_ref().unwrap())
        .enumerate()
        .filter(|e| e.1.size.unwrap() == smallest_encoding)
        .collect::<Vec<_>>();
    

    // Expect only a single remaining encoding
    if encodings.len() > 1
    {
        let mut candidate_notes = Vec::new();

        for encoding in encodings
        {
            candidate_notes.push(build_recursive_candidate_note(
                0,
                &instr.matches[encoding.0],
                decls,
                defs));
        }

        report.push_parent(
            "multiple matches with the same encoding size",
            &ast_instr.span);

        report.push_multiple(candidate_notes);

        report.pop_parent();

        return Err(());
    }


    let chosen_encoding = encodings[0].1.clone();

    let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
    instr.chosen_encoding = Some(chosen_encoding);

    Ok(asm2::ResolutionState::Resolved)
}


fn build_recursive_candidate_note(
    depth: usize,
    instr_match: &asm2::InstructionMatch,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs)
    -> diagn::Message
{
    let ruledef = &defs.ruledefs.get(instr_match.ruledef_ref);
    let rule = &ruledef.get_rule(instr_match.rule_ref);

    let ruledef_name =
        &decls.ruledefs.get(instr_match.ruledef_ref).name;

    let mut msg = {
        if depth == 0
        {
            diagn::Message::note_span(
                format!(
                    "match on `{}`, rule {}:",
                    ruledef_name,
                    instr_match.rule_ref.0),
                &rule.pattern_span)
        }
        else
        {
            diagn::Message::note_span(
                format!(
                    "nested match on `{}`, rule {}:",
                    ruledef_name,
                    instr_match.rule_ref.0),
                &rule.pattern_span)
        }
    };

    msg.short_excerpt = true;

    for arg in &instr_match.args
    {
        if let asm2::InstructionArgumentKind::Nested(ref nested_match) = arg.kind
        {
            msg.inner.push(build_recursive_candidate_note(
                depth + 1,
                &nested_match,
                decls,
                defs));
        }
    }

    msg
}


fn resolve_instruction_match<'symbol_ctx>(
    report: &mut diagn::Report,
    mtch: &asm2::InstructionMatch,
    decls: &'symbol_ctx asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    symbol_ctx: &'symbol_ctx util::SymbolContext)
    -> Result<expr::Value, ()>
{
    let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
    let rule = &ruledef.get_rule(mtch.rule_ref);

    let mut eval_ctx = expr::EvalContext2::new();

    for (index, arg) in mtch.args.iter().enumerate()
    {
        match arg.kind
        {
            asm2::InstructionArgumentKind::Expr(ref expr) =>
            {
                let mut arg_value = asm2::resolver::eval(
                    report,
                    decls,
                    defs,
                    symbol_ctx,
                    &mut expr::EvalContext2::new(),
                    &expr)?;

                if arg_value.is_unknown()
                {
                    return Ok(expr::Value::Unknown);
                }

                let param = &rule.parameters[index];

                check_and_constrain_argument(
                    report,
                    &expr.span(),
                    &mut arg_value,
                    param.typ)?;

                eval_ctx.set_local(
                    &param.name,
                    arg_value);
                
                eval_ctx.set_token_sub(
                    &param.name,
                    arg.tokens.clone());
            }

            asm2::InstructionArgumentKind::Nested(ref nested_match) =>
            {
                let arg_value = resolve_instruction_match(
                    report,
                    &nested_match,
                    decls,
                    defs,
                    symbol_ctx)?;

                let param = &rule.parameters[index];

                eval_ctx.set_local(
                    &param.name,
                    arg_value);
                
                eval_ctx.set_token_sub(
                    &param.name,
                    arg.tokens.clone());
            }
        }
    }

    asm2::resolver::eval(
        report,
        decls,
        defs,
        symbol_ctx,
        &mut eval_ctx,
        &rule.expr)
}


pub fn check_and_constrain_argument(
    report: &mut diagn::Report,
    span: &diagn::Span,
    value: &mut expr::Value,
    typ: asm2::RuleParameterType)
    -> Result<(), ()>
{
    let bigint = value.expect_bigint_mut(report, span)?;

    match typ
    {
        asm2::RuleParameterType::Unspecified =>
            Ok(()),
            
        asm2::RuleParameterType::Unsigned(size) =>
        {
            check_argument_for_integer_type(
                report,
                span,
                size,
                "u",
                bigint,
                |x| x.sign() == -1 ||
                    x.min_size() > size)
        }

        asm2::RuleParameterType::Signed(size) =>
        {
            check_argument_for_integer_type(
                report,
                span,
                size,
                "s",
                bigint,
                |x| (x.sign() == 0 && size == 0) ||
                    (x.sign() == 1 && x.min_size() >= size) ||
                    (x.sign() == -1 && x.min_size() > size))
        }

        asm2::RuleParameterType::Integer(size) =>
        {
            check_argument_for_integer_type(
                report,
                span,
                size,
                "i",
                bigint,
                |x| x.min_size() > size)
        }

        asm2::RuleParameterType::RuledefRef(_) =>
            unreachable!(),
    }
}


fn check_argument_for_integer_type(
    report: &mut diagn::Report,
    span: &diagn::Span,
    size: usize,
    typename_prefix: &'static str,
    bigint: &mut util::BigInt,
    check_fn: impl Fn(&mut util::BigInt) -> bool)
    -> Result<(), ()>
{
    if check_fn(bigint)
    {
        report.error_span(
            format!(
                "argument out of range for type `{}{}`",
                typename_prefix,
                size),
            span);
        
        Err(())
    }
    else
    {
        bigint.size = Some(size);
        Ok(())
    }
}