use crate::*;

pub fn eval_asm(
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalAsmBlockQuery,
) -> Result<expr::Value, ()>
{
    query
        .eval_ctx
        .check_recursion_depth_limit(query.report, query.span)?;

    // Keep the current position to advance
    // between instructions
    let mut cur_position = ctx.bank_data.cur_position;

    let mut result = util::BigInt::new(0, Some(0));

    let ast = asm::parser::parse(query.report, query.tokens)?;

    for node in &ast.nodes
    {
        if let asm::AstAny::Instruction(ast_instr) = node
        {
            let substs = parse_substitutions(query.report, &ast_instr.tokens)?;

            let new_tokens = perform_substitutions(&ast_instr.tokens, &substs, query)?;

            // Run the matcher algorithm
            let mut matches = asm::matcher::match_instr(opts, defs, &new_tokens);

            let attempted_match_excerpt = {
                if substs.len() == 0
                {
                    None
                }
                else
                {
                    Some(format!(
                        "match attempted: `{}`",
                        new_tokens.iter().map(|t| t.text()).collect::<String>()
                    ))
                }
            };

            if let Some(ref excerpt) = attempted_match_excerpt
            {
                query
                    .report
                    .push_parent_short_note(excerpt.clone(), ast_instr.span);
            }

            let maybe_no_matches =
                asm::matcher::error_on_no_matches(query.report, ast_instr.span, &matches);

            if let Some(_) = attempted_match_excerpt
            {
                query.report.pop_parent();
            }

            maybe_no_matches?;

            // Clone the context to use our own position
            let new_bank_datum = asm::resolver::BankData { cur_position };

            let mut inner_ctx = ctx.clone();
            inner_ctx.bank_data = &new_bank_datum;

            // Try to resolve the encoding
            let mut new_eval_ctx = query.eval_ctx.hygienize_locals_for_asm_subst();

            if let Some(ref s) = attempted_match_excerpt
            {
                query.report.push_parent_short_note(s, ast_instr.span);
            }

            let maybe_encodings = asm::resolver::instruction::resolve_encoding(
                query.report,
                opts,
                ast_instr.span,
                fileserver,
                &mut matches,
                decls,
                defs,
                &mut inner_ctx,
                &mut new_eval_ctx,
            );

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

                result = result.concat((result.size.unwrap(), 0), &encodings[0].1, (size, 0));
            }
            else if !ctx.can_guess()
            {
                return Err(());
            }
        }
        else
        {
            query
                .report
                .error_span("invalid content for `asm` block", node.span());

            return Err(());
        }
    }

    Ok(expr::Value::make_integer(result))
}

struct AsmSubstitution<'a>
{
    pub start: usize,
    pub end: usize,
    pub name: &'a str,
    pub span: diagn::Span,
}

fn parse_substitutions<'tokens>(
    report: &mut diagn::Report,
    tokens: &'tokens [syntax::Token],
) -> Result<Vec<AsmSubstitution<'tokens>>, ()>
{
    let mut substs = Vec::new();

    let mut walker = syntax::TokenWalker::new(tokens);

    while !walker.is_over()
    {
        if let Some(_) = walker.maybe_expect(syntax::TokenKind::BraceOpen)
        {
            let start = walker.get_previous_token_index();

            let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;

            let name = tk_name.excerpt.as_ref().unwrap();
            let span = tk_name.span;

            walker.expect(report, syntax::TokenKind::BraceClose)?;

            let end = walker.get_previous_token_index() + 1;

            substs.push(AsmSubstitution {
                start,
                end,
                name,
                span,
            });
        }
        else
        {
            walker.advance();
        }
    }

    Ok(substs)
}

fn perform_substitutions<'tokens>(
    tokens: &'tokens [syntax::Token],
    substs: &Vec<AsmSubstitution<'tokens>>,
    info: &mut expr::EvalAsmBlockQuery,
) -> Result<Vec<syntax::Token>, ()>
{
    let mut result: Vec<syntax::Token> = Vec::new();

    let mut copied_up_to = 0;

    for subst in substs
    {
        while copied_up_to < subst.start
        {
            result.push(tokens[copied_up_to].clone());
            copied_up_to += 1;
        }

        let token_subst = {
            match info.eval_ctx.get_token_subst(subst.name)
            {
                Some(t) => t,
                None =>
                {
                    info.report.error_span(
                        format!("unknown substitution argument `{}`", subst.name),
                        subst.span,
                    );

                    return Err(());
                }
            }
        };

        for token in token_subst.iter()
        {
            let mut new_token = token.clone();
            new_token.span = subst.span;
            result.push(new_token);
        }

        copied_up_to += subst.end - subst.start;
    }

    while copied_up_to < tokens.len()
    {
        result.push(tokens[copied_up_to].clone());
        copied_up_to += 1;
    }

    Ok(result)
}
