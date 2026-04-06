use crate::*;


struct AsmBlockLabel
{
    pub name: String,
    pub value: expr::Value,
}


struct AsmBlockInstruction<'opts>
{
    pub src: String,
    pub substs: Vec<AsmSubstitution>,
    pub eval_ctx: expr::EvalContext<'opts>,
    pub matches: asm::InstructionMatches,
    pub encoding: expr::Value,
    pub resolved: bool,
}


pub fn eval_asm(
    fileserver: &mut dyn util::FileServer,
    opts: &asm::AssemblyOptions,
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
    let position_at_start_resolved = ctx.bank_data.cur_position_resolved;


    let mut labels = Vec::new();
    let mut instrs = Vec::new();

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

            asm::check_reserved_name(
                query.report,
                ast_symbol.decl_span,
                query.eval_ctx.opts,
                ast_symbol.name.as_ref())?;

            labels.push(AsmBlockLabel {
                name: ast_symbol.name.clone(),
                value: expr::Value::make_unknown(),
            });
        }

        else if let asm::AstAny::Instruction(ast_instr) = node
        {
            let substs = parse_substitutions(
                query.report,
                ast_instr.span,
                &ast_instr.src)?;

            let mut new_eval_ctx = expr::EvalContext::new_deepened(
                query.eval_ctx);

            let new_excerpt = perform_substitutions(
                &ast_instr.src,
                &substs,
                &mut new_eval_ctx,
                query)?;
            
            let matches = asm::matcher::match_instr(
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
                        &new_excerpt))
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
            
            instrs.push(AsmBlockInstruction {
                src: new_excerpt,
                substs,
                eval_ctx: new_eval_ctx,
                matches,
                encoding: expr::Value::make_unknown(),
                resolved: false,
            });
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
        opts,
        decls,
        defs,
        ctx,
        query,
        position_at_start,
        position_at_start_resolved,
        &mut labels,
        &mut instrs)
}


fn resolve_iteratively(
    fileserver: &mut dyn util::FileServer,
    opts: &asm::AssemblyOptions,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalAsmBlockQuery,
    position_at_start: usize,
    position_at_start_resolved: bool,
    labels: &mut Vec<AsmBlockLabel>,
    instrs: &mut Vec<AsmBlockInstruction>)
    -> Result<expr::Value, ()>
{
    let mut iter_count = 0;
    let max_iterations = query.eval_ctx.opts.max_iterations;

    while iter_count < max_iterations
    {
        iter_count += 1;

        if opts.debug_iterations
        {
            println!(
                "{} [== asm block `{:?}` iteration {} ==]",
                " ".repeat(query.eval_ctx.get_recursion_depth()),
                query.span,
                iter_count);
        }

        let is_first_iteration = iter_count == 1;
        let is_last_iteration =
            iter_count == max_iterations &&
            ctx.is_last_iteration;

        let result = resolve_once(
            fileserver,
            opts,
            decls,
            defs,
            ctx,
            query,
            position_at_start,
            position_at_start_resolved,
            labels,
            instrs,
            is_first_iteration,
            is_last_iteration)?;

        if opts.debug_iterations
        {
            println!(
                "{} asm block `{:?}` result = {}",
                " ".repeat(query.eval_ctx.get_recursion_depth()),
                query.span,
                result.resolution.debug_label());
        }

        if result.resolution.is_resolved() &&
            opts.optimize_statically_known
        {
            return Ok(result.value);
        }

        if result.resolution.is_stable_or_resolved()
        {
            break;
        }
    
        if ctx.is_first_iteration &&
            ctx.can_guess()
        {
            return Ok(expr::Value::make_unknown());
        }
    }

    iter_count += 1;

    if opts.debug_iterations
    {
        println!(
            "{} [== asm block iteration {} (final) ==]",
            " ".repeat(query.eval_ctx.get_recursion_depth()),
            iter_count);
    }

    let result = resolve_once(
        fileserver,
        opts,
        decls,
        defs,
        ctx,
        query,
        position_at_start,
        position_at_start_resolved,
        labels,
        instrs,
        false,
        ctx.is_last_iteration)?;

    if opts.debug_iterations
    {
        println!(
            "{} asm block `{:?}` result = {}",
            " ".repeat(query.eval_ctx.get_recursion_depth()),
            query.span,
            result.resolution.debug_label());
    }

    if result.resolution.is_stable_or_resolved()
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
    resolution: asm::ResolutionState,
}


fn resolve_once(
    fileserver: &mut dyn util::FileServer,
    _opts: &asm::AssemblyOptions,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalAsmBlockQuery,
    position_at_start: usize,
    position_at_start_resolved: bool,
    labels: &mut Vec<AsmBlockLabel>,
    instrs: &mut Vec<AsmBlockInstruction>,
    is_first_iteration: bool,
    is_last_iteration: bool)
    -> Result<AsmBlockResult, ()>
{
    let mut result_metadata = expr::ValueMetadata::new().statically_known();
    let mut result = util::BigInt::new(0, Some(0));
    let mut cur_position = position_at_start;
    let mut cur_position_resolved = position_at_start_resolved;
    let mut resolution = asm::ResolutionState::Resolved;

    let mut cur_label = 0;
    let mut cur_instr = 0;
    
    for node in &query.ast.nodes
    {
        // Clone the context to use our own position
        let new_bank_datum = asm::resolver::BankData {
            cur_position,
            cur_position_resolved,
        };

        let mut inner_ctx = ctx.clone();
        inner_ctx.bank_data = &new_bank_datum;
        inner_ctx.is_first_iteration = is_first_iteration;
        inner_ctx.is_last_iteration = is_last_iteration;


        if let asm::AstAny::Symbol(ast_symbol) = node
        {
            let label = &labels[cur_label];

            let cur_address = inner_ctx.eval_address(
                query.report,
                ast_symbol.decl_span,
                defs,
                inner_ctx.can_guess())?;

            let is_stable = cur_address.is_stable(&label.value);
            let mut is_resolved = false;

            let label_resolution = asm::resolver::handle_value_resolution(
                query.eval_ctx.opts,
                query.report,
                ast_symbol.decl_span,
                inner_ctx.can_guess(),
                cur_address.is_guess(),
                is_stable,
                &mut is_resolved,
                true,
                "label",
                "label address",
                Some(&ast_symbol.name),
                &cur_address)?;

            resolution.merge(label_resolution);

            let label = &mut labels[cur_label];
            label.value = cur_address;

            cur_label += 1;
        }

        else if let asm::AstAny::Instruction(ast_instr) = node
        {
            let instr = &instrs[cur_instr];

            let mut new_eval_ctx = instr.eval_ctx.clone();

            for label in labels.iter()
            {
                new_eval_ctx.set_local(
                    label.name.clone(),
                    label.value.clone());
            }

            let attempted_match_excerpt = {
                if instr.substs.len() == 0
                {
                    None
                }
                else
                {
                    Some(format!(
                        "match attempted: `{}`",
                        &instr.src))
                }
            };

            if let Some(ref s) = attempted_match_excerpt
            {
                query.report.push_parent_short_note(
                    s,
                    ast_instr.span);
            }

            // Extract matches to satisfy the borrow checker
            let instr = &mut instrs[cur_instr];
            let mut matches = std::mem::replace(
                &mut instr.matches,
                asm::InstructionMatches::new());

            let mut new_encoding = expr::Value::make_unknown();
            let mut is_resolved = false;
            
            let instr_resolution = asm::resolver::instruction::resolve_instruction_inner(
                query.report,
                ast_instr.span,
                query.eval_ctx.opts,
                fileserver,
                decls,
                defs,
                &instr.src,
                &mut matches,
                &instr.encoding,
                &mut new_encoding,
                &mut is_resolved,
                true,
                &mut inner_ctx,
                &mut new_eval_ctx);
                
            if let Some(_) = attempted_match_excerpt
            {
                query.report.pop_parent();
            }

            let instr_resolution = instr_resolution?;

            let instr = &mut instrs[cur_instr];
            instr.matches = matches;
            instr.resolved = is_resolved;
            instr.encoding = new_encoding;

            resolution.merge(instr_resolution);

            // Add the encoding to the result value
            // and advance the position
            if let expr::Value::Integer(_, ref bigint) = instr.encoding
            {
                let size = bigint.size.unwrap();

                cur_position += size;
                cur_position_resolved &= instr.resolved;

                result = result.concat(
                    (result.size.unwrap(), 0),
                    bigint,
                    (size, 0));

                result_metadata.mark_derived_from(instr.encoding.get_metadata());
            }
            else 
            {
                result_metadata.mark_guess();
                cur_position_resolved &= false;

                if !inner_ctx.can_guess()
                {
                    return Err(());
                }
            }
            
            cur_instr += 1;
        }

        else
        {
            unreachable!();
        }
    }

    Ok(AsmBlockResult {
        value: expr::Value::make_integer(result)
            .statically_known()
            .with_metadata(result_metadata),
        resolution,
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


fn perform_substitutions<'src, 'opts>(
    excerpt: &'src str,
    substs: &Vec<AsmSubstitution>,
    new_eval_ctx: &mut expr::EvalContext<'opts>,
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
            if let Some(t) = info.eval_ctx.get_token_subst(&subst.name)
            {
                t
            }
            else if let Some(value) = info.eval_ctx.get_local(&subst.name)
            {
                let new_name = new_eval_ctx.new_asm_subst();
                new_eval_ctx.set_local(&new_name, value);
                std::borrow::Cow::Owned(new_name)
            }
            else
            {
                info.report.error_span(
                    format!(
                        "unknown substitution argument `{}`",
                        subst.name),
                    subst.span);
                
                return Err(());
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
