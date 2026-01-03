use crate::*;


pub fn eval_asm(
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalAsmBlockQuery)
    -> Result<expr::Value, ()>
{
    query.eval_ctx.check_recursion_depth_limit(
        query.report,
        query.span)?;


    // Keep the current position to advance
    // between instructions
    let position_at_start = ctx.bank_data.cur_position;


    let mut labels = std::collections::HashMap::<String, expr::Value>::new();
    for node in &query.ast.nodes
    {
        if let asm::AstAny::Symbol(ast_symbol) = node
        {
            if !matches!(ast_symbol.kind, asm::AstSymbolKind::Label)
            {
                query.report.error_span(
                    "only labels are permitted in `asm` blocks",
                    node.span());
    
                return Err(());
            }

            if ast_symbol.hierarchy_level != 0
            {
                query.report.error_span(
                    "only top-level labels are permitted in `asm` blocks",
                    node.span());
    
                return Err(());
            }

            labels.insert(
                ast_symbol.name.clone(),
                expr::Value::make_unknown());
        }

        else if let asm::AstAny::Instruction(_) = node
        {
            continue;
        }

        else
        {
            query.report.error_span(
                "invalid content for `asm` block",
                node.span());

            return Err(());
        }
    }

    resolve_iteratively(
        fileserver,
        decls,
        defs,
        ctx,
        query,
        position_at_start,
        &mut labels)
}


fn resolve_iteratively(
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalAsmBlockQuery,
    position_at_start: usize,
    labels: &mut std::collections::HashMap::<String, expr::Value>)
    -> Result<expr::Value, ()>
{
    let mut iter_count = 0;
    let max_iterations = query.eval_ctx.opts.max_iterations;

    while iter_count < max_iterations
    {
        iter_count += 1;

        let is_first_iteration = iter_count == 1;
        let is_last_iteration =
            iter_count == max_iterations &&
            ctx.is_last_iteration;

        let result = resolve_once(
            fileserver,
            decls,
            defs,
            ctx,
            query,
            position_at_start,
            labels,
            is_first_iteration,
            is_last_iteration)?;

        if !result.unstable
        {
            break;
        }
    }

    let result = resolve_once(
        fileserver,
        decls,
        defs,
        ctx,
        query,
        position_at_start,
        labels,
        false,
        ctx.is_last_iteration)?;

    if !result.unstable
    {
        return Ok(result.value);
    }
    
    if ctx.can_guess()
    {
        return Ok(expr::Value::make_unknown());
    }

    query.report.error_span(
        "`asm` block did not converge",
        query.span);

    return Err(());
}


struct AsmBlockResult
{
    value: expr::Value,
    unstable: bool,
}


fn resolve_once(
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalAsmBlockQuery,
    position_at_start: usize,
    labels: &mut std::collections::HashMap::<String, expr::Value>,
    is_first_iteration: bool,
    is_last_iteration: bool)
    -> Result<AsmBlockResult, ()>
{
    let mut result = util::BigInt::new(0, Some(0));
    let mut cur_position = position_at_start;
    let mut unstable = false;
    
    for node in &query.ast.nodes
    {
        // Clone the context to use our own position
        let new_bank_datum = asm::resolver::BankData {
            cur_position,
        };

        let mut inner_ctx = ctx.clone();
        inner_ctx.bank_data = &new_bank_datum;
        inner_ctx.is_first_iteration = is_first_iteration;
        inner_ctx.is_last_iteration = is_last_iteration;


        if let asm::AstAny::Symbol(ast_symbol) = node
        {
            let cur_address = inner_ctx.eval_address(
                query.report,
                ast_symbol.decl_span,
                defs,
                inner_ctx.can_guess())?;

            let new_value = expr::Value::make_integer(cur_address);
            
            let maybe_prev_value = labels.get(&ast_symbol.name);
            if let Some(prev_value) = maybe_prev_value
            {
                if prev_value != &new_value
                {
                    unstable = true;
                }
            }

            labels.insert(
                ast_symbol.name.clone(),
                new_value);
        }

        else if let asm::AstAny::Instruction(ast_instr) = node
        {
            let substs = parse_substitutions(
                query.report,
                ast_instr.span,
                &ast_instr.src)?;

            let new_excerpt = perform_substitutions(
                &ast_instr.src,
                &substs,
                query)?;

            
            // Run the matcher algorithm
            let mut matches = asm::matcher::match_instr(
                query.eval_ctx.opts,
                defs,
                ast_instr.span,
                &new_excerpt);


            let attempted_match_excerpt = {
                if substs.len() == 0
                {
                    None
                }
                else
                {
                    Some(format!(
                        "match attempted: `{}`",
                        new_excerpt))
                }
            };

            if let Some(ref excerpt) = attempted_match_excerpt
            {
                query.report.push_parent_short_note(
                    excerpt.clone(),
                    ast_instr.span);
            }
                    
            let maybe_no_matches = asm::matcher::error_on_no_matches(
                query.report,
                ast_instr.span,
                &matches);

            if let Some(_) = attempted_match_excerpt
            {
                query.report.pop_parent();
            }

            maybe_no_matches?;
            

            // Try to resolve the encoding
            let mut new_eval_ctx = query.eval_ctx
                .hygienize_locals_for_asm_subst();

            for (label_name, label_value) in labels.iter()
            {
                new_eval_ctx.set_local(label_name, label_value.clone());
            }

            if let Some(ref s) = attempted_match_excerpt
            {
                query.report.push_parent_short_note(
                    s,
                    ast_instr.span);
            }
            
            let maybe_encodings = asm::resolver::instruction::resolve_encoding(
                query.report,
                query.eval_ctx.opts,
                ast_instr.span,
                fileserver,
                &mut matches,
                decls,
                defs,
                &mut inner_ctx,
                &mut new_eval_ctx);
                
            if let Some(_) = attempted_match_excerpt
            {
                query.report.pop_parent();
            }

            // Add the encoding to the result value
            // and advance the position
            if let Some(encodings) = maybe_encodings?
            {
                let size = encodings[0].1.size.unwrap();

                cur_position += size;

                result = result.concat(
                    (result.size.unwrap(), 0),
                    &encodings[0].1,
                    (size, 0));
            }
            else 
            {
                unstable = true;

                if !inner_ctx.can_guess()
                {
                    return Err(());
                }
            }
        }
    }

    Ok(AsmBlockResult {
        value: expr::Value::make_integer(result),
        unstable,
    })
}


struct AsmSubstitution
{
    pub start: usize,
    pub end: usize,
    pub name: String,
    pub span: diagn::Span,
}


fn parse_substitutions<'excerpt>(
    report: &mut diagn::Report,
    span: diagn::Span,
    excerpt: &'excerpt str)
    -> Result<Vec<AsmSubstitution>, ()>
{
    let mut substs = Vec::new();

    let mut walker = syntax::Walker::new(
        excerpt,
        span.file_handle,
        span.location().unwrap().0);

    while !walker.is_over()
    {
        walker.skip_ignorable();

        if let Some(tk_brace_open) = walker.maybe_expect(syntax::TokenKind::BraceOpen)
        {
            let start = walker.get_index_at_span_start(
                tk_brace_open.span);
            
            let tk_name = walker.expect(
                report,
                syntax::TokenKind::Identifier)?;
            
            let name = walker.get_span_excerpt(tk_name.span).to_string();
            let span = tk_name.span;

            walker.expect(
                report,
                syntax::TokenKind::BraceClose)?;
            
            let end = walker.get_cursor_index();
            
            substs.push(AsmSubstitution {
                start,
                end,
                name,
                span,
            });
        }
        else
        {
            walker.advance_to_token_end(&walker.next_token());
        }
    }

    Ok(substs)
}


fn perform_substitutions<'src>(
    excerpt: &'src str,
    substs: &Vec<AsmSubstitution>,
    info: &mut expr::EvalAsmBlockQuery)
    -> Result<String, ()>
{
    let mut result = String::new();

    let mut copied_up_to = 0;

    for subst in substs
    {
        if copied_up_to < subst.start
        {
            result.push_str(&excerpt[copied_up_to..subst.start]);
            copied_up_to = subst.start;
        }
        
        let subst_str = {
            match info.eval_ctx.get_token_subst(&subst.name)
            {
                Some(t) => t,
                None =>
                {
                    info.report.error_span(
                        format!(
                            "unknown substitution argument `{}`",
                            subst.name),
                        subst.span);
                    
                    return Err(());
                }
            }
        };

        result.push_str(&subst_str);

        copied_up_to += subst.end - subst.start;
    }

    if copied_up_to < excerpt.len()
    {
        result.push_str(&excerpt[copied_up_to..]);
    }

    Ok(result)
}
