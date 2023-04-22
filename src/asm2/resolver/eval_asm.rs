use crate::*;


pub fn eval_asm(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalAsmInfo2)
    -> Result<expr::Value, ()>
{
    info.eval_ctx.check_recursion_depth_limit(
        info.report,
        info.span)?;


    // Keep the current position to advance
    // between instructions
    let mut cur_position = ctx.bank_data.cur_position;


    let mut result = util::BigInt::new(0, Some(0));
    
    let ast = asm2::parser::parse(
        info.report,
        info.tokens)?;

    for node in &ast.nodes
    {
        if let asm2::AstAny::Instruction(ast_instr) = node
        {
            let substs = parse_substitutions(
                info.report,
                &ast_instr.tokens)?;

            let new_tokens = perform_substitutions(
                &ast_instr.tokens,
                &substs,
                info)?;

            
            // Run the matcher algorithm
            let mut matches = asm2::matcher::match_instr(
                defs,
                &new_tokens);

            asm2::matcher::error_on_no_matches(
                info.report,
                &ast_instr.span,
                &matches)?;
            
            
            // Clone the context to use our own position
            let new_bank_datum = asm2::resolver::BankData {
                cur_position,
            };

            let mut inner_ctx = ctx.clone();
            inner_ctx.bank_data = &new_bank_datum;


            // Try to resolve the encoding
            let mut new_eval_ctx = info.eval_ctx
                .hygienize_locals_for_asm_subst();

            let maybe_encoding = asm2::resolver::instruction::resolve_encoding(
                info.report,
                &ast_instr.span,
                fileserver,
                &mut matches,
                decls,
                defs,
                &mut inner_ctx,
                &mut new_eval_ctx)?;


            // Add the encoding to the result value
            // and advance the position
            if let Some(encoding) = maybe_encoding
            {
                cur_position += encoding.size.unwrap();

                result = result.concat(
                    (result.size.unwrap(), 0),
                    &encoding,
                    (encoding.size.unwrap(), 0));
            }
        }

        else
        {
            info.report.error_span(
                "invalid content for `asm` block",
                node.span());

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
    pub span: &'a diagn::Span,
}


fn parse_substitutions<'tokens>(
    report: &mut diagn::Report,
    tokens: &'tokens [syntax::Token])
    -> Result<Vec<AsmSubstitution<'tokens>>, ()>
{
    let mut substs = Vec::new();

    let mut walker = syntax::TokenWalker::new(tokens);

    while !walker.is_over()
    {
        if let Some(_) = walker.maybe_expect(syntax::TokenKind::BraceOpen)
        {
            let start = walker.get_previous_token_index();

            let tk_name = walker.expect(
                report,
                syntax::TokenKind::Identifier)?;
            
            let name = tk_name.excerpt.as_ref().unwrap();
            let span = &tk_name.span;

            walker.expect(
                report,
                syntax::TokenKind::BraceClose)?;
            
            let end = walker.get_current_token_index();
            
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
    info: &mut expr::EvalAsmInfo2)
    -> Result<Vec<syntax::Token>, ()>
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
                        format!(
                            "unknown substitution argument `{}`",
                            subst.name),
                        subst.span);
                    
                    return Err(());
                }
            }
        };

        for token in token_subst.iter()
        {
            let mut new_token = token.clone();
            new_token.span = subst.span.clone();
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
