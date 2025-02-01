use crate::*;


pub fn resolve_instruction(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast_instr: &asm::AstInstruction,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    ctx: &asm::ResolverContext)
    -> Result<asm::ResolutionState, ()>
{
    let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());

    if instr.resolved
    {
        return Ok(asm::ResolutionState::Resolved);
    }

    // Extract matches to satisfy the borrow checker
    let mut matches = std::mem::replace(
        &mut instr.matches,
        asm::InstructionMatches::new());
        
    let maybe_encodings = resolve_encoding(
        report,
        opts,
        ast_instr.span,
        fileserver,
        &mut matches,
        decls,
        defs,
        ctx,
        &mut expr::EvalContext::new())?;

    let has_any_matches =
        maybe_encodings.is_some();

    let has_single_match =
        maybe_encodings.as_ref().map_or(false, |e| e.len() == 1);

    let maybe_chosen_encoding =
        maybe_encodings.as_ref().map(|e| e[0].1.clone());

    // Reassign matches to satisfy the borrow checker
    let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
    instr.matches = matches;


    // Check for stable resolution
    let is_stable =
        Some(&instr.encoding) == maybe_chosen_encoding.as_ref();


    // Update the instruction's encoding if available
    if let Some(encoding) = maybe_chosen_encoding
    {
        instr.encoding = encoding;

        // Optimize future iterations for the case where it's
        // statically known that the encoding can be resolved
        // in the first pass
        if opts.optimize_statically_known &&
            ctx.is_first_iteration &&
            instr.encoding_statically_known &&
            has_single_match
        {
            if opts.debug_iterations
            {
                println!("instr: {} = {:?} [static]",
                    ast_instr.src,
                    instr.encoding);
            }

            instr.resolved = true;
            return Ok(asm::ResolutionState::Resolved);
        }
    }

    
    if !is_stable
    {
        // On the final iteration, unstable guesses become errors.
        // If encodings came out None, an inner error has already been reported.
        if ctx.is_last_iteration && has_any_matches
        {
            report.error_span(
                "instruction encoding did not converge",
                ast_instr.span);
        }
        
        if opts.debug_iterations
        {
            println!("instr: {} = {:?}",
                ast_instr.src,
                instr.encoding);
        }
        
        return Ok(asm::ResolutionState::Unresolved);
    }


    Ok(asm::ResolutionState::Resolved)
}


pub fn resolve_encoding<'encoding>(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    instr_span: diagn::Span,
    fileserver: &mut dyn util::FileServer,
    matches: &'encoding mut asm::InstructionMatches,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    arg_eval_ctx: &mut expr::EvalContext)
    -> Result<Option<Vec<(usize, &'encoding util::BigInt)>>, ()>
{
    report.push_parent_cap();

    report.push_parent(
        "failed to resolve instruction",
        instr_span);
    
    // Try to resolve every match
    let resolved = resolve_instruction_matches(
        report,
        opts,
        fileserver,
        matches,
        decls,
        defs,
        ctx,
        arg_eval_ctx);

    report.pop_parent();
    report.pop_parent_cap();
    resolved?;


    // Print FailedConstraint error messages
    // if no match succeeded
    let num_encodings_resolved = matches
        .iter()
        .filter(|m| m.encoding.is_resolved())
        .count();

    if num_encodings_resolved == 0
    {
        if !ctx.can_guess()
        {
            let mut msgs = Vec::new();

            for mtch in matches
            {
                let encoding = &mtch.encoding;

                if let asm::InstructionMatchResolution::FailedConstraint(ref msg) = encoding
                {
                    msgs.push(msg.clone());
                }
            }
            
            report.message(
                diagn::Message::fuse_topmost(msgs));
        }

        return Ok(None);
    }


    // Retain only encodings which are Resolved,
    // and keep their original indices
    let encodings_resolved = matches
        .iter()
        .enumerate()
        .filter(|m| m.1.encoding.is_resolved())
        .map(|m| (m.0, m.1.encoding.unwrap_resolved()))
        .collect::<Vec<_>>();

    // Now only retain the smallest encodings
    let smallest_size = encodings_resolved
        .iter()
        .map(|e| e.1.size.unwrap())
        .min()
        .unwrap();

    let smallest_encodings = encodings_resolved
        .iter()
        .filter(|e| e.1.size.unwrap() == smallest_size)
        .copied()
        .collect::<Vec<_>>();
    

    // Expect only a single remaining encoding
    // on the last iteration
    if !ctx.can_guess() &&
        smallest_encodings.len() > 1
    {
        let mut notes = Vec::new();

        for encoding in smallest_encodings
        {
            notes.push(build_recursive_candidate_note(
                0,
                &matches[encoding.0],
                decls,
                defs));
        }

        report.push_parent(
            "multiple matches with the same encoding size",
            instr_span);

        report.push_multiple(notes);

        report.pop_parent();

        return Ok(None);
    }


    return Ok(Some(smallest_encodings));
}


fn resolve_instruction_matches(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    matches: &mut asm::InstructionMatches,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    arg_eval_ctx: &mut expr::EvalContext)
    -> Result<(), ()>
{
    for index in 0..matches.len()
    {
        let mtch = &matches[index];
        let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
        let rule = &ruledef.get_rule(mtch.rule_ref);

        let value = resolve_instruction_match(
            report,
            opts,
            &mtch,
            fileserver,
            decls,
            defs,
            ctx,
            arg_eval_ctx)?;

        let value_definite = value.expect_error_or_sized_bigint(
            report,
            rule.expr.returned_value_span())?;


        if let expr::Value::Integer(bigint) = value_definite
        {
            matches[index].encoding =
                asm::InstructionMatchResolution::Resolved(bigint);
        }
        else if let expr::Value::FailedConstraint(msg) = value_definite
        {
            matches[index].encoding =
                asm::InstructionMatchResolution::FailedConstraint(msg);
        }
        else
        {
            matches[index].encoding =
                asm::InstructionMatchResolution::Unresolved;
        }
    }

    Ok(())
}


fn build_recursive_candidate_note(
    depth: usize,
    instr_match: &asm::InstructionMatch,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs)
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
                rule.pattern_span)
        }
        else
        {
            diagn::Message::short_note_span(
                format!(
                    "nested match on `{}`, rule {}:",
                    ruledef_name,
                    instr_match.rule_ref.0),
                rule.pattern_span)
        }
    };

    for arg in &instr_match.args
    {
        if let asm::InstructionArgumentKind::Nested(ref nested_match) = arg.kind
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
    opts: &asm::AssemblyOptions,
    mtch: &asm::InstructionMatch,
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    arg_eval_ctx: &mut expr::EvalContext)
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
        rule.pattern_span);

    let maybe_value = resolve_instruction_match_inner(
        report,
        opts,
        &mtch,
        fileserver,
        decls,
        defs,
        ctx,
        arg_eval_ctx);

    report.pop_parent();

    maybe_value
}


fn resolve_instruction_match_inner(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    mtch: &asm::InstructionMatch,
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    arg_eval_ctx: &mut expr::EvalContext)
    -> Result<expr::Value, ()>
{
    let ruledef = defs.ruledefs.get(mtch.ruledef_ref);
    let rule = &ruledef.get_rule(mtch.rule_ref);

    let mut eval_ctx = expr::EvalContext::new_deepened(
        &arg_eval_ctx);

    for (index, arg) in mtch.args.iter().enumerate()
    {
        match arg.kind
        {
            asm::InstructionArgumentKind::Expr(ref expr) =>
            {
                let arg_value = asm::resolver::eval(
                    report,
                    opts,
                    fileserver,
                    decls,
                    defs,
                    ctx,
                    arg_eval_ctx,
                    &expr)?;

                if arg_value.should_propagate()
                {
                    return Ok(arg_value);
                }

                let param = &rule.parameters[index];

                let constrained_arg_value = check_and_constrain_argument(
                    report,
                    expr.span(),
                    arg_value,
                    param.typ)?;

                eval_ctx.set_local(
                    &param.name,
                    constrained_arg_value);
                
                eval_ctx.set_token_subst(
                    &param.name,
                    arg.excerpt.clone());
            }

            asm::InstructionArgumentKind::Nested(ref nested_match) =>
            {
                let arg_value = resolve_instruction_match(
                    report,
                    opts,
                    &nested_match,
                    fileserver,
                    decls,
                    defs,
                    ctx,
                    arg_eval_ctx)?;

                if arg_value.should_propagate()
                {
                    return Ok(arg_value);
                }

                let param = &rule.parameters[index];

                eval_ctx.set_local(
                    &param.name,
                    arg_value);
                
                eval_ctx.set_token_subst(
                    &param.name,
                    arg.excerpt.clone());
            }
        }
    }

    let mut rule_ctx = (*ctx).clone();
    rule_ctx.file_handle_ctx = Some(rule.expr.span().file_handle);

    asm::resolver::eval(
        report,
        opts,
        fileserver,
        decls,
        defs,
        &rule_ctx,
        &mut eval_ctx,
        &rule.expr)
}


pub fn check_and_constrain_argument(
    report: &mut diagn::Report,
    span: diagn::Span,
    value: expr::Value,
    typ: asm::RuleParameterType)
    -> Result<expr::Value, ()>
{
    let bigint = value
        .coallesce_to_integer()
        .expect_bigint(report, span)?
        .to_owned();

    match typ
    {
        asm::RuleParameterType::Unspecified =>
            Ok(value),
            
        asm::RuleParameterType::Unsigned(size) =>
        {
            check_and_constrain_value_for_integer_type(
                report,
                span,
                size,
                "u",
                bigint,
                |x| x.sign() == -1 ||
                    x.min_size() > size)
        }

        asm::RuleParameterType::Signed(size) =>
        {
            check_and_constrain_value_for_integer_type(
                report,
                span,
                size,
                "s",
                bigint,
                |x| (x.sign() == 0 && size == 0) ||
                    (x.sign() == 1 && x.min_size() >= size) ||
                    (x.sign() == -1 && x.min_size() > size))
        }

        asm::RuleParameterType::Integer(size) =>
        {
            check_and_constrain_value_for_integer_type(
                report,
                span,
                size,
                "i",
                bigint,
                |x| x.min_size() > size)
        }

        asm::RuleParameterType::RuledefRef(_) =>
            unreachable!(),
    }
}


fn check_and_constrain_value_for_integer_type(
    report: &mut diagn::Report,
    span: diagn::Span,
    size: usize,
    typename_prefix: &'static str,
    mut bigint: util::BigInt,
    failure_check: impl Fn(&util::BigInt) -> bool)
    -> Result<expr::Value, ()>
{
    if failure_check(&bigint)
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