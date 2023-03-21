use crate::*;


pub fn resolve_instruction(
    report: &mut diagn::Report,
    ast_instr: &asm2::AstInstruction,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    // Update the instruction's address if available
    if let Some(addr) = ctx.bank_data.cur_position
    {
        let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
        
        if instr.position_within_bank.is_none()
        {
            instr.position_within_bank = Some(addr);
        }
    }


    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());
    
    // Skip this instruction if already resolved
    if let Some(_) = instr.encoding
    {
        return Ok(asm2::ResolutionState::Resolved);
    }


    let (is_guess, maybe_encoding) = resolve_encoding(
        report,
        ast_instr,
        decls,
        defs,
        ctx)?;

    if let Some(encoding) = maybe_encoding
    {
        let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
        
        // In the final iteration, the current guess should be
        // stable with respect to the previously guessed value
        if ctx.is_last_iteration && is_guess
        {
            if Some(&encoding) != instr.encoding_guess.as_ref()
            {
                dbg!(&instr);
                report.error_span(
                    "instruction encoding did not converge",
                    &ast_instr.span);
            }
        }

        
        if ctx.is_last_iteration || !is_guess
        {
            instr.encoding_size = Some(encoding.size.unwrap());
            instr.encoding = Some(encoding);
        }
        else
        {
            instr.encoding_size_guess = Some(encoding.size.unwrap());
            instr.encoding_guess = Some(encoding);
        }
    }


    Ok(asm2::ResolutionState::Resolved)
}


fn resolve_encoding(
    report: &mut diagn::Report,
    ast_instr: &asm2::AstInstruction,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<(bool, Option<util::BigInt>), ()>
{
    // Try to resolve every match
    resolve_instruction_matches(
        report,
        ast_instr,
        decls,
        defs,
        ctx)?;


    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());

    let has_no_guess = instr.matches
        .iter()
        .all(|m| m.encoding.is_resolved_or_failed());

    // Use definite encodings if available,
    // or fallback to the encoding guesses
    let encodings = instr.matches
        .iter()
        .map(|m|
        {
            if m.encoding.is_resolved_or_failed()
                { &m.encoding }
            else if m.encoding_guess.is_resolved_or_failed()
                { &m.encoding_guess }
            else
                { &asm2::InstructionMatchResolution::Unresolved }
        })
        .collect::<Vec<_>>();
    

    // Retain only encodings which aren't FailedConstraint
    let encodings_within_constraints = encodings
        .iter()
        .enumerate()
        .filter(|m| m.1.is_resolved())
        .map(|m| (m.0, m.1.unwrap_resolved()))
        .collect::<Vec<_>>();

    
    // Print FailedConstraint error messages
    // if no match succeeded
    if encodings_within_constraints.len() == 0
    {
        if ctx.is_last_iteration
        {
            let mut msgs = Vec::new();

            for mtch in &instr.matches
            {
                let encoding = {
                    if mtch.encoding.is_resolved_or_failed()
                        { &mtch.encoding }
                    else if mtch.encoding_guess.is_resolved_or_failed()
                        { &mtch.encoding_guess }
                    else
                        { &asm2::InstructionMatchResolution::Unresolved }
                };

                if let asm2::InstructionMatchResolution::FailedConstraint(ref msg) = encoding
                {
                    msgs.push(msg.clone());
                }
            }
            
            report.message(
                diagn::Message::fuse_topmost(msgs));
        }

        return Ok((false, None));
    }


    // Only retain the smallest encodings
    let smallest_size = encodings_within_constraints
        .iter()
        .min_by_key(|e| e.1.size.unwrap())
        .unwrap()
        .1.size
        .unwrap();

    let smallest_encodings = encodings_within_constraints
        .iter()
        .filter(|e| e.1.size.unwrap() == smallest_size)
        .collect::<Vec<_>>();
    

    // Expect only a single remaining encoding
    if smallest_encodings.len() > 1 &&
        (ctx.is_last_iteration || has_no_guess)
    {
        let mut candidate_notes = Vec::new();

        for encoding in smallest_encodings
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


    let chosen_encoding = smallest_encodings[0].1.clone();

    return Ok((
        !has_no_guess,
        Some(chosen_encoding)));
}


fn resolve_instruction_matches(
    report: &mut diagn::Report,
    ast_instr: &asm2::AstInstruction,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<(), ()>
{
    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());

    // Try to resolve every match
    for index in 0..instr.matches.len()
    {
        let instr = defs.instructions.get(ast_instr.item_ref.unwrap());
        let mtch = &instr.matches[index];
        let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
        let rule = &ruledef.get_rule(mtch.rule_ref);


        // Skip this match if already resolved
        if let asm2::InstructionMatchResolution::Resolved(_) = &mtch.encoding
        {
            continue;
        }
        else if let asm2::InstructionMatchResolution::FailedConstraint(_) = &mtch.encoding
        {
            continue;
        }


        if ctx.is_first_iteration
        {
            let value_definite = {
                report.push_parent(
                    "failed to resolve instruction",
                    &ast_instr.span);
                
                let maybe_value = resolve_instruction_match(
                    report,
                    &mtch,
                    decls,
                    defs,
                    ctx,
                    false);

                let maybe_value = maybe_value
                    .and_then(|v| v.expect_error_or_sized_bigint(
                        report,
                        &rule.expr.returned_value_span()));
        
                report.pop_parent();
                report.pop_parent();

                maybe_value?
            };


            if let expr::Value::Integer(bigint) = value_definite
            {
                let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());

                instr.matches[index].encoding =
                    asm2::InstructionMatchResolution::Resolved(bigint);

                continue;
            }
            else if let expr::Value::FailedConstraint(msg) = value_definite
            {
                let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());

                instr.matches[index].encoding =
                    asm2::InstructionMatchResolution::FailedConstraint(msg);

                continue;
            }
        }


        let maybe_guess = resolve_instruction_match(
            report,
            &mtch,
            decls,
            defs,
            ctx,
            true);

        let maybe_guess = maybe_guess
            .and_then(|v| v.expect_error_or_sized_bigint(
                report,
                &rule.expr.returned_value_span()));
            
        let guess = {
            match maybe_guess
            {
                Ok(guess) => guess,
                Err(()) => continue,
            }
        };


        let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());

        if let expr::Value::Integer(guess_bigint) = guess
        {
            instr.matches[index].encoding_size_guess =
                guess_bigint.size.unwrap();

            instr.matches[index].encoding_guess =
                asm2::InstructionMatchResolution::Resolved(guess_bigint);
        }
        else if let expr::Value::FailedConstraint(guess_msg) = guess
        {
            instr.matches[index].encoding_guess =
                asm2::InstructionMatchResolution::FailedConstraint(guess_msg);
        }
    }

    Ok(())
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
            diagn::Message::short_note_span(
                format!(
                    "match on `{}`, rule {}:",
                    ruledef_name,
                    instr_match.rule_ref.0),
                &rule.pattern_span)
        }
        else
        {
            diagn::Message::short_note_span(
                format!(
                    "nested match on `{}`, rule {}:",
                    ruledef_name,
                    instr_match.rule_ref.0),
                &rule.pattern_span)
        }
    };

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


fn resolve_instruction_match(
    report: &mut diagn::Report,
    mtch: &asm2::InstructionMatch,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    can_guess: bool)
    -> Result<expr::Value, ()>
{
    let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
    let rule = &ruledef.get_rule(mtch.rule_ref);
    let ruledef_name = &decls.ruledefs.get(mtch.ruledef_ref).name;

    report.push_parent_short_note(
        format!(
            "within `{}`, rule {}",
            ruledef_name,
            mtch.rule_ref.0),
        &rule.pattern_span);

    let maybe_value = resolve_instruction_match_inner(
        report,
        &mtch,
        decls,
        defs,
        ctx,
        can_guess);

    report.pop_parent();

    maybe_value
}


fn resolve_instruction_match_inner(
    report: &mut diagn::Report,
    mtch: &asm2::InstructionMatch,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    can_guess: bool)
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
                let arg_value = asm2::resolver::eval(
                    report,
                    decls,
                    defs,
                    ctx,
                    &mut expr::EvalContext2::new(),
                    can_guess,
                    &expr)?;

                if arg_value.should_propagate()
                {
                    return Ok(arg_value);
                }

                let param = &rule.parameters[index];

                let constrained_arg_value = check_and_constrain_argument(
                    report,
                    &expr.span(),
                    arg_value,
                    param.typ)?;

                eval_ctx.set_local(
                    &param.name,
                    constrained_arg_value);
                
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
                    ctx,
                    can_guess)?;

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
        ctx,
        &mut eval_ctx,
        can_guess,
        &rule.expr)
}


pub fn check_and_constrain_argument(
    report: &mut diagn::Report,
    span: &diagn::Span,
    value: expr::Value,
    typ: asm2::RuleParameterType)
    -> Result<expr::Value, ()>
{
    let bigint = value.expect_bigint(report, span)?.clone();

    match typ
    {
        asm2::RuleParameterType::Unspecified =>
            Ok(expr::Value::make_integer(bigint)),
            
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
    mut bigint: util::BigInt,
    check_fn: impl Fn(&util::BigInt) -> bool)
    -> Result<expr::Value, ()>
{
    if check_fn(&bigint)
    {
        let msg = diagn::Message::error_span(
            format!(
                "argument out of range for type `{}{}`",
                typename_prefix,
                size),
            span);
        
        Ok(expr::Value::FailedConstraint(
            report.wrap_in_parents(msg)))
    }
    else
    {
        bigint.size = Some(size);
        Ok(expr::Value::make_integer(bigint))
    }
}