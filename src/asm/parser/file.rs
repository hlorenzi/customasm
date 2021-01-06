use crate::*;


pub fn parse_file<TFilename: Into<String>>(
    report: diagn::RcReport,
    asm_state: &mut asm::State,
    fileserver: &dyn util::FileServer,
    filename: TFilename,
    span: Option<&diagn::Span>,
    parsed_filenames: &mut std::collections::HashSet<String>)
    -> Result<(), ()>
{
    let filename = filename.into();
    let chars = fileserver.get_chars(report.clone(), &filename, span)?;
    let tokens = syntax::tokenize(report.clone(), &filename, &chars)?;
    let parser = syntax::Parser::new(Some(report.clone()), &tokens);

    parsed_filenames.insert(filename.clone());
    
    let mut state = asm::parser::State
    {
        report,
        asm_state,
        fileserver,
        filename: std::rc::Rc::new(filename.clone()),
        parser,
        parsed_filenames,
    };

    //println!("{:#?}", state.parser.tokens.iter().map(|t| t.kind).collect::<Vec<_>>());

    while !state.parser.is_over()
    {
        parse_line(&mut state)?;
    }
	
    parsed_filenames.remove(&filename);
	Ok(())
}


pub fn parse_line(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    if state.parser.next_is(0, syntax::TokenKind::Hash)
    {
        parse_directive(state)?;
    }
    else if state.parser.next_is(0, syntax::TokenKind::Identifier) &&
        state.parser.next_is(1, syntax::TokenKind::Colon)
    {
        asm::parser::parse_symbol(state)?;
    }
    else if state.parser.next_is(0, syntax::TokenKind::Identifier) &&
        state.parser.next_is(1, syntax::TokenKind::Equal)
    {
        asm::parser::parse_symbol(state)?;
    }
    else if state.parser.next_is(0, syntax::TokenKind::Dot)
    {
        asm::parser::parse_symbol(state)?;
    }
    else if state.parser.maybe_expect_linebreak().is_some()
    {
        return Ok(());
    }
    else
    {
        asm::parser::parse_rule_invocation(state)?;
    }

    Ok(())
}


pub fn parse_directive(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_hash = state.parser.expect(syntax::TokenKind::Hash)?;
    let tk_directive = state.parser.expect(syntax::TokenKind::Identifier)?;
    let directive = tk_directive.excerpt.as_ref().unwrap().to_ascii_lowercase();

    let mut parsed_data_directive = false;

    if directive.chars().next() == Some('d')
    {
        if directive == "d"
        {
            asm::parser::parse_directive_data(state, None, &tk_hash)?;
            parsed_data_directive = true;
        }
        else if let Ok(elem_size) = usize::from_str_radix(&directive[1..], 10)
        {
            if elem_size > 0
            {
                asm::parser::parse_directive_data(state, Some(elem_size), &tk_hash)?;
                parsed_data_directive = true;
            }
        }
    }
    
    if !parsed_data_directive
    {
        match directive.as_ref()
        {
            "bits" => asm::parser::parse_directive_bits(state)?,
            "bankdef" => asm::parser::parse_directive_bankdef(state)?,
            "bank" => asm::parser::parse_directive_bank(state)?,
            "ruledef" | "cpudef" => asm::parser::parse_directive_ruledef(state, &tk_directive, true)?,
            "subruledef" | "tokendef" => asm::parser::parse_directive_ruledef(state, &tk_directive, false)?,
            "include" => asm::parser::parse_directive_include(state)?,
            "res" => asm::parser::parse_directive_res(state)?,
            "align" => asm::parser::parse_directive_align(state)?,
            "labelalign" => asm::parser::parse_directive_labelalign(state)?,
            "addr" => asm::parser::parse_directive_addr(state)?,
            //"enable" => asm::parser::parse_directive_enable(state)?,
            _ =>
            {
                state.report.error_span(
                    "unknown directive",
                    &tk_hash.span.join(&tk_directive.span));
                return Err(());
            }
        }
    }

    state.parser.expect_linebreak()
}


pub fn parse_expr_bigint(state: &mut asm::parser::State) -> Result<(util::BigInt, diagn::Span), ()>
{
    let expr = expr::Expr::parse(&mut state.parser)?;
    let value = state.asm_state.eval_expr(
        state.report.clone(),
        &expr,
        &state.asm_state.get_ctx(&state),
        &mut expr::EvalContext::new(),
        state.fileserver,
        true)?;

    match value.get_bigint()
    {
        Some(bigint) => Ok((bigint, expr.span())),
        None =>
        {
            state.report.error_span("expected integer value", &expr.span());
            Err(())
        }
    }
}


pub fn parse_expr_usize(state: &mut asm::parser::State) -> Result<usize, ()>
{
    let expr = expr::Expr::parse(&mut state.parser)?;
    let value = state.asm_state.eval_expr(
        state.report.clone(),
        &expr,
        &state.asm_state.get_ctx(&state),
        &mut expr::EvalContext::new(),
        state.fileserver,
        true)?;

    match value.get_bigint()
    {
        Some(bigint) =>
        {
            match bigint.checked_to_usize()
            {
                Some(value) => Ok(value),
                None =>
                {
                    state.report.error_span("value is outside of valid range", &expr.span());
                    Err(())
                }
            }
        }
        None =>
        {
            state.report.error_span("expected integer value", &expr.span());
            Err(())
        }
    }
}


pub fn parse_expr_usize_fn<F>(state: &mut asm::parser::State, func: F) -> Result<usize, ()>
where F: Fn(usize) -> Option<usize>
{
    let expr = expr::Expr::parse(&mut state.parser)?;
    let value = state.asm_state.eval_expr(
        state.report.clone(),
        &expr,
        &state.asm_state.get_ctx(&state),
        &mut expr::EvalContext::new(),
        state.fileserver,
        true)?;

    match value.get_bigint()
    {
        Some(bigint) =>
        {
            match bigint.checked_to_usize()
            {
                Some(value) =>
                {
                    match func(value)
                    {
                        Some(value_transf) => Ok(value_transf),
                        None =>
                        {
                            state.report.error_span("value is outside of valid range", &expr.span());
                            Err(())
                        }
                    }
                }
                None =>
                {
                    state.report.error_span("value is outside of valid range", &expr.span());
                    Err(())
                }
            }
        }
        None =>
        {
            state.report.error_span("expected integer value", &expr.span());
            Err(())
        }
    }
}