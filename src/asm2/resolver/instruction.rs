use crate::*;


pub fn resolve_instruction(
    report: &mut diagn::Report,
    ast_instr: &asm2::AstInstruction,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<asm2::ResolutionState, ()>
{
    let maybe_encoding = resolve_encoding(
        report,
        ast_instr,
        decls,
        defs,
        ctx)?;


    let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());


    // Check for stable resolution
    let is_stable =
        Some(&instr.encoding) == maybe_encoding.as_ref();


    // Update the instruction's address
    instr.position_within_bank = ctx.bank_data.cur_position;
    

    // Update the instruction's encoding if available
    if let Some(ref encoding) = maybe_encoding
    {
        instr.encoding = encoding.clone();
    }

    
    if !is_stable
    {
        // On the final iteration, unstable guesses become errors.
        // If encoding is Some, an inner error has already been reported.
        if ctx.is_last_iteration && !maybe_encoding.is_some()
        {
            report.error_span(
                "instruction encoding did not converge",
                &ast_instr.span);
        }
        
        println!("instr: {} = {:?}",
            ast_instr.tokens.iter().map(|t| t.text()).collect::<Vec<_>>().join(""),
            instr.encoding);
        return Ok(asm2::ResolutionState::Unresolved);
    }


    Ok(asm2::ResolutionState::Resolved)
}


fn resolve_encoding(
    report: &mut diagn::Report,
    ast_instr: &asm2::AstInstruction,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<Option<util::BigInt>, ()>
{
    // Try to resolve every match
    resolve_instruction_matches(
        report,
        ast_instr,
        decls,
        defs,
        ctx)?;


    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());

    // Retain only encodings which are Resolved
    let encodings_resolved = instr.matches
        .iter()
        .enumerate()
        .filter(|m| m.1.encoding.is_resolved())
        .map(|m| (m.0, m.1.encoding.unwrap_resolved()))
        .collect::<Vec<_>>();

    
    // Print FailedConstraint error messages
    // if no match succeeded
    if encodings_resolved.len() == 0
    {
        if ctx.is_last_iteration
        {
            let mut msgs = Vec::new();

            for mtch in &instr.matches
            {
                let encoding = &mtch.encoding;

                if let asm2::InstructionMatchResolution::FailedConstraint(ref msg) = encoding
                {
                    msgs.push(msg.clone());
                }
            }
            
            report.message(
                diagn::Message::fuse_topmost(msgs));
        }

        return Ok(None);
    }


    // Only retain the smallest encodings
    let smallest_size = encodings_resolved
        .iter()
        .map(|e| e.1.size.unwrap())
        .min()
        .unwrap();

    let smallest_encodings = encodings_resolved
        .iter()
        .filter(|e| e.1.size.unwrap() == smallest_size)
        .collect::<Vec<_>>();
    

    // Expect only a single remaining encoding
    // on the last iteration
    if ctx.is_last_iteration && smallest_encodings.len() > 1
    {
        let mut notes = Vec::new();

        for encoding in smallest_encodings
        {
            notes.push(build_recursive_candidate_note(
                0,
                &instr.matches[encoding.0],
                decls,
                defs));
        }

        report.push_parent(
            "multiple matches with the same encoding size",
            &ast_instr.span);

        report.push_multiple(notes);

        report.pop_parent();

        return Ok(None);
    }


    let chosen_encoding = smallest_encodings[0].1.clone();

    return Ok(Some(chosen_encoding));
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
                true);

            let maybe_value = maybe_value
                .and_then(|v| v.expect_error_or_sized_bigint(
                    report,
                    &rule.expr.returned_value_span()));
    
            report.pop_parent();
            report.pop_parent();

            maybe_value?
        };


        let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());

        if let expr::Value::Integer(bigint) = value_definite
        {
            instr.matches[index].encoding =
                asm2::InstructionMatchResolution::Resolved(bigint);
        }
        else if let expr::Value::FailedConstraint(msg) = value_definite
        {
            instr.matches[index].encoding =
                asm2::InstructionMatchResolution::FailedConstraint(msg);
        }
        else
        {
            instr.matches[index].encoding =
                asm2::InstructionMatchResolution::Unresolved;
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
    failure_fn: impl Fn(&util::BigInt) -> bool)
    -> Result<expr::Value, ()>
{
    if failure_fn(&bigint)
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