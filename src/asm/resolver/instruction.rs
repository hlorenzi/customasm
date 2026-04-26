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

    if instr.resolved && opts.optimize_statically_known {
        return Ok(asm::ResolutionState::Resolved);
    }

    // Extract matches to satisfy the borrow checker
    let mut matches = std::mem::replace(
        &mut instr.matches,
        asm::InstructionMatches::new());

    let prev_encoding = instr.encoding.clone();
    let mut new_encoding = expr::Value::make_unknown();
    let mut is_resolved = false;

    let resolution = resolve_instruction_inner(
        report,
        ast_instr.span,
        opts,
        fileserver,
        decls,
        defs,
        &ast_instr.src,
        &mut matches,
        &prev_encoding,
        &mut new_encoding,
        &mut is_resolved,
        false,
        ctx,
        &mut expr::EvalContext::new(opts))?;
        
    // Reassign matches to satisfy the borrow checker
    let instr = defs.instructions.get_mut(ast_instr.item_ref.unwrap());
    instr.matches = matches;
    instr.resolved = is_resolved;
    instr.encoding = new_encoding;

    Ok(resolution)
}


pub fn resolve_instruction_inner(
    report: &mut diagn::Report,
    span: diagn::Span,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    src: &str,
    matches: &mut Vec<asm::InstructionMatch>,
    prev_encoding: &expr::Value,
    new_encoding: &mut expr::Value,
    is_resolved: &mut bool,
    suppress_diagn: bool,
    ctx: &asm::ResolverContext,
    arg_eval_ctx: &mut expr::EvalContext)
    -> Result<asm::ResolutionState, ()>
{
    let maybe_encodings = resolve_encoding(
        report,
        opts,
        span,
        fileserver,
        matches,
        decls,
        defs,
        ctx,
        arg_eval_ctx)?;

    let suppress_diagn =
        suppress_diagn ||
        maybe_encodings.is_none();

    let has_single_match =
        maybe_encodings.as_ref().map_or(false, |e| e.len() == 1);

    let maybe_chosen_encoding =
        maybe_encodings.as_ref().map(|e| e[0].1.clone());

    // Check for stable resolution
    let is_stable = maybe_chosen_encoding
        .as_ref()
        .map_or(false, |e| e.is_stable(prev_encoding));

    if let Some(encoding) = maybe_chosen_encoding
    {
        if !has_single_match
        {
            *new_encoding = encoding.as_guess();
        }
        else
        {
            *new_encoding = encoding;
        }
    }

    asm::resolver::handle_value_resolution(
        opts,
        report,
        span,
        ctx.can_guess(),
        new_encoding.is_guess() || !has_single_match,
        is_stable,
        is_resolved,
        suppress_diagn,
        "instr",
        "instruction encoding",
        Some(src),
        &new_encoding)
}


pub fn finalize_instruction<'instr>(
    _report: &mut diagn::Report,
    _span: diagn::Span,
    instr: &'instr asm::Instruction)
    -> Result<&'instr util::BigInt, ()>
{
    let expr::Value::Integer(_, ref encoding) = instr.encoding
        else { unreachable!() };
    
    // Definite size was checked in resolve_instruction_matches
    Ok(encoding)
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
    -> Result<Option<Vec<(usize, &'encoding expr::Value)>>, ()>
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

                if let asm::InstructionMatchResolution::FailedConstraint(_, msg) = encoding
                {
                    msgs.push(msg.clone());
                }
                else
                {
                    msgs.push(diagn::Message::error_span(
                        "instruction encoding did not converge",
                        instr_span));
                }
            }
            
            report.message(
                diagn::Message::fuse_topmost(msgs));
        }

        return Ok(None);
    }

    
    let num_encodings_known = matches
        .iter()
        .filter(|m| m.encoding.is_resolved() && !m.encoding.unwrap_resolved().should_propagate())
        .count();

    if num_encodings_known == 0
    {
        if !ctx.can_guess()
        {
            let mut msgs = Vec::new();

            for _ in matches
            {
                msgs.push(diagn::Message::error_span(
                    "instruction encoding did not converge",
                    instr_span));
            }
            
            report.message(
                diagn::Message::fuse_topmost(msgs));
        }

        return Ok(None);
    }

    // Mark all encodings as guesses if any single one is a guess
    let any_encoding_guess = matches
        .iter()
        .any(|m| m.encoding.is_guess());

    if any_encoding_guess
    {
        for mtch in matches.iter_mut()
        {
            if let asm::matcher::InstructionMatchResolution::Resolved(ref mut resolved) = mtch.encoding
            {
                resolved.mark_guess();
            }
            else if let asm::matcher::InstructionMatchResolution::FailedConstraint(ref mut metadata, _) = mtch.encoding
            {
                metadata.mark_guess();
            }
        }
    }

    // Retain only encodings which are Resolved,
    // and keep their original indices
    let encodings_resolved = matches
        .iter()
        .enumerate()
        .filter(|m| m.1.encoding.is_resolved())
        .map(|m| (m.0, m.1.encoding.unwrap_resolved()))
        .filter(|m| matches!(m.1, expr::Value::Integer(_, _)))
        .collect::<Vec<_>>();

    // Now only retain the smallest encodings
    let smallest_size = encodings_resolved
        .iter()
        .map(|e| e.1.unwrap_bigint().size.unwrap())
        .min()
        .unwrap();

    let smallest_encodings = encodings_resolved
        .iter()
        .filter(|e| e.1.unwrap_bigint().size.unwrap() == smallest_size)
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

        if let expr::Value::FailedConstraint(metadata, msg) = value_definite
        {
            matches[index].encoding =
                asm::InstructionMatchResolution::FailedConstraint(metadata, msg);
        }
        else
        {
            matches[index].encoding =
                asm::InstructionMatchResolution::Resolved(value_definite);
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
                    instr_match.rule_ref.get_raw()),
                rule.pattern_span)
        }
        else
        {
            diagn::Message::short_note_span(
                format!(
                    "nested match on `{}`, rule {}:",
                    ruledef_name,
                    instr_match.rule_ref.get_raw()),
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
            mtch.rule_ref.get_raw()),
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
                    fileserver,
                    opts,
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

                if !arg.excerpt.contains(expr::ASM_SUBSTITUTION_VARIABLE)
                {
                    eval_ctx.set_token_subst(
                        &param.name,
                        arg.excerpt.clone());
                }
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
                
                if !arg.excerpt.contains(expr::ASM_SUBSTITUTION_VARIABLE)
                {
                    eval_ctx.set_token_subst(
                        &param.name,
                        arg.excerpt.clone());
                }
            }
        }
    }

    let mut rule_ctx = (*ctx).clone();
    rule_ctx.file_handle_ctx = Some(rule.expr.span().file_handle);

    Ok(asm::resolver::eval(
        report,
        fileserver,
        opts,
        decls,
        defs,
        &rule_ctx,
        &mut eval_ctx,
        &rule.expr)?)
}


pub fn check_and_constrain_argument(
    report: &mut diagn::Report,
    span: diagn::Span,
    value: expr::Value,
    typ: asm::RuleParameterType)
    -> Result<expr::Value, ()>
{
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
                value,
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
                value,
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
                value,
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
    mut value: expr::Value,
    failure_check: impl Fn(&util::BigInt) -> bool)
    -> Result<expr::Value, ()>
{
    let bigint = value
        .expect_bigint(report, span)?;

    if failure_check(&bigint)
    {
        let msg = diagn::Message::error_span(
            format!(
                "argument is out of range for type `{}{}`",
                typename_prefix,
                size),
            span);
        
        Ok(expr::Value::FailedConstraint(
            expr::ValueMetadata::new(),
            report.wrap_in_parents(msg)))
    }
    else
    {
        let expr::Value::Integer(_, ref mut bigint) = value
            else { unreachable!() };

        bigint.size = Some(size);
        Ok(value)
    }
}