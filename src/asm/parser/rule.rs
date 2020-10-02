use crate::*;


pub fn parse_rule(
    state: &mut asm::parser::State)
    -> Result<asm::Rule, ()>
{
    let mut rule = asm::Rule::new();

    while !state.parser.next_is(0, syntax::TokenKind::HeavyArrowRight)
    {
        let tk = state.parser.advance();
        
        if tk.kind == syntax::TokenKind::BraceOpen
        {
            let tk_param_name = state.parser.expect(syntax::TokenKind::Identifier)?;
            let param_name = tk_param_name.excerpt.as_ref().unwrap().clone();

            let param_type = if let Some(_) = state.parser.maybe_expect(syntax::TokenKind::Colon)
            {
                let tk_typename = state.parser.expect(syntax::TokenKind::Identifier)?;
                let typename = tk_typename.excerpt.as_ref().unwrap().clone();
                interpret_typename(state, &typename, &tk_typename.span)?
            }
            else
            {
                asm::PatternParameterType::Unspecified
            };

            state.parser.expect(syntax::TokenKind::BraceClose)?;

            rule.pattern_add_parameter(asm::PatternParameter
            {
                name: param_name,
                typ: param_type,
            });
        }
        
        else if tk.kind.is_allowed_pattern_token()
        {
            rule.pattern_add_exact(&tk);
        }
        
        else
        {
            state.report.error_span("invalid pattern token", &tk.span);
            return Err(());
        }
    }

    let tk_heavy_arrow = state.parser.expect(syntax::TokenKind::HeavyArrowRight)?;

    if rule.pattern.len() == 0
    {
        state.report.error_span("expected pattern", &tk_heavy_arrow.span.before());
        return Err(());
    }


    rule.production = expr::Expr::parse(&mut state.parser)?;

    /*if !rule.production.has_size()
    {
        state.report.error_span(
            "size of rule production must be known; \
            try using a bit slice like `x[hi:lo]`",
            &rule.production.span());

        return Err(());
    }*/

    Ok(rule)
}


fn interpret_typename(
    state: &mut asm::parser::State,
    typename: &str,
    span: &diagn::Span)
    -> Result<asm::PatternParameterType, ()>
{
    let first_char = typename.chars().next();

    if first_char == Some('u') ||
        first_char == Some('s') ||
        first_char == Some('i')
    {
        if let Ok(size) = usize::from_str_radix(&typename[1..], 10)
        {
            match first_char
            {
                Some('u') => return Ok(asm::PatternParameterType::Unsigned(size)),
                Some('s') => return Ok(asm::PatternParameterType::Signed(size)),
                Some('i') => return Ok(asm::PatternParameterType::Integer(size)),
                _ => unreachable!()
            }
        }
    }
    
    let rule_group_ref = state.asm_state.find_ruleset(
        typename,
        state.report.clone(),
        &span)?;

    Ok(asm::PatternParameterType::Ruleset(rule_group_ref))
}