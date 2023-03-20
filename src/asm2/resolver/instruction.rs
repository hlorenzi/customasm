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
    if let Some(addr) = ctx.bank_data.cur_address
    {
        let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
        
        if instr.address_from_bank_start.is_none()
        {
            instr.address_from_bank_start = Some(addr);
        }
    }


    // Attempt to resolve the instruction's encoding
    // if it's still Unknown
    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());
    
    if let None = instr.chosen_encoding
    {
        let (maybe_size, is_guess, maybe_encoding) = resolve_encoding(
            report,
            ast_instr,
            decls,
            defs,
            ctx)?;

        if let Some(encoding) = maybe_encoding
        {
            let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
            instr.encoding_size = Some(encoding.size.unwrap());
            instr.chosen_encoding = Some(encoding);
        }
        else if let Some(size) = maybe_size
        {
            let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
            if is_guess
            {
                instr.encoding_size_guess = Some(size);
            }
            else
            {
                instr.encoding_size = Some(size);
            }
        }
    }


    // Check for resolution
    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());

    if instr.encoding_size.is_none() ||
        instr.chosen_encoding.is_none()
    {
        if ctx.is_final_iteration
        {
            report.error_span(
                "instruction encoding did not converge",
                &ast_instr.span);
        }

        Ok(asm2::ResolutionState::Unresolved)
    }
    else
    {
        Ok(asm2::ResolutionState::Resolved)
    }
}


fn resolve_encoding(
    report: &mut diagn::Report,
    ast_instr: &asm2::AstInstruction,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    ctx: &asm2::ResolverContext)
    -> Result<(Option<usize>, bool, Option<util::BigInt>), ()>
{
    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());
    let mut candidate_reports = vec![None; instr.matches.len()];

    // Try to resolve every match
    for index in 0..instr.matches.len()
    {
        let instr = defs.instructions.get(ast_instr.item_ref.unwrap());
        let mtch = &instr.matches[index];
        let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
        let rule = &ruledef.get_rule(mtch.rule_ref);
        let ruledef_name = &decls.ruledefs.get(mtch.ruledef_ref).name;


        // Skip this match if already resolved
        if let asm2::InstructionMatchResolution::Resolved(_) = &mtch.resolved
        {
            continue;
        }
        else if let asm2::InstructionMatchResolution::FailedConstraint = &mtch.resolved
        {
            continue;
        }


        let value = {
            let mut candidate_report = diagn::Report::new();

            candidate_report.push_parent(
                "failed to resolve instruction",
                &ast_instr.span);
            
            candidate_report.push_parent_short_note(
                format!(
                    "within `{}`, rule {}",
                    ruledef_name,
                    mtch.rule_ref.0),
                &rule.pattern_span);

            let maybe_value = resolve_instruction_match(
                &mut candidate_report,
                &mtch,
                decls,
                defs,
                ctx,
                false);

            candidate_report.pop_parent();
            candidate_report.pop_parent();

            if candidate_report.has_errors()
            {
                match maybe_value
                {
                    Ok(expr::Value::FailedConstraint) => {}
                    _ => {},//candidate_report.transfer_to(report),
                }
            }

            candidate_reports[index] = Some(candidate_report);
            maybe_value?
        };


        if let expr::Value::Unknown = value
        {
            let mut guess_report = diagn::Report::new();

            let maybe_guess = resolve_instruction_match(
                &mut guess_report,
                &mtch,
                decls,
                defs,
                ctx,
                true);

            println!("{:#?}", defs);
            println!("instruction match #{}, guess = {:?}", index, maybe_guess);

            let guess = {
                match maybe_guess
                {
                    Err(()) => continue,
                    Ok(guess) => guess,
                }
            };

            let guess_bigint = {
                match guess
                {
                    expr::Value::Integer(bigint) if bigint.size.is_some() =>
                        bigint,
                    _ => continue,
                }
            };

            let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
            
            instr.matches[index].encoding_size_guess =
                Some(guess_bigint.size.unwrap());

            continue;
        }


        if let expr::Value::FailedConstraint = value
        {
            let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
            
            instr.matches[index].resolved =
                asm2::InstructionMatchResolution::FailedConstraint;

            println!("FailedConstraint");
            continue;
        }

            
        let bigint = {
            report.push_parent(
                "failed to resolve instruction",
                &ast_instr.span);
            
            report.push_parent_short_note(
                format!(
                    "within `{}`, rule {}",
                    ruledef_name,
                    mtch.rule_ref.0),
                &rule.pattern_span);

            let maybe_bigint = value.expect_sized_bigint(
                report,
                &rule.expr.returned_value_span());

            report.pop_parent();
            report.pop_parent();

            maybe_bigint?
        };


        let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
        
        instr.matches[index].resolved =
            asm2::InstructionMatchResolution::Resolved(bigint.clone());
    }


    let instr = defs.instructions.get(ast_instr.item_ref.unwrap());

    // Stop if any matches are still Unknown,
    // but try to estimate the encoding size if every match
    // outputs the same size
    if instr.matches.len() != 0 &&
        instr.matches.iter().any(|m| matches!(
            m.resolved,
            asm2::InstructionMatchResolution::Unresolved))
    {
        let encoding_sizes = instr.matches
            .iter()
            .map(|m| m.encoding_size);

        let mut instr_size = None;

        for size in encoding_sizes
        {
            if let Some(size) = size
            {
                if let Some(prev_size) = instr_size
                {
                    if size != prev_size
                    {
                        instr_size = None;
                        break;
                    }
                }
                
                instr_size = Some(size);
            }
        }

        if let Some(instr_size) = instr_size
        {
            return Ok((Some(instr_size), false, None));
        }


        let encoding_size_guess = instr.matches
            .iter()
            .map(|m| m.encoding_size_guess)
            .filter(|g| g.is_some())
            .map(|g| g.unwrap())
            .min();

        if let Some(guess) = encoding_size_guess
        {
            return Ok((Some(guess), true, None));
        }

        return Ok((None, false, None));
    }


    // Retain only encodings which aren't FailedConstraint
    let encodings_within_constraints = instr.matches
        .iter()
        .enumerate()
        .filter(|m| match m.1.resolved {
            asm2::InstructionMatchResolution::Resolved(_) => true,
            _ => false,
        })
        .map(|m| match m.1.resolved {
            asm2::InstructionMatchResolution::Resolved(ref r) => (m.0, r),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();


    // Only retain the smallest encodings
    let smallest_encoding = encodings_within_constraints
        .iter()
        .min_by_key(|e| e.1.size.unwrap())
        .unwrap()
        .1.size
        .unwrap();

    let encodings = encodings_within_constraints
        .iter()
        .copied()
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
    Ok((
        Some(chosen_encoding.size.unwrap()),
        false,
        Some(chosen_encoding)))
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
        report.error_span(
            format!(
                "argument out of range for type `{}{}`",
                typename_prefix,
                size),
            span);
        
        Ok(expr::Value::FailedConstraint)
    }
    else
    {
        bigint.size = Some(size);
        Ok(expr::Value::make_integer(bigint))
    }
}