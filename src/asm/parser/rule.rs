use crate::*;


pub fn parse_rule(
    state: &mut asm::parser::State)
    -> Result<asm::Rule, ()>
{
    let mut rule = asm::Rule::new();

    while !state.parser.next_is(0, syntax::TokenKind::HeavyArrowRight)
    {
        let tk = state.parser.advance();
        
        let is_beginning_of_pattern = rule.pattern.len() == 0;

        if is_beginning_of_pattern
        {
            if tk.kind.is_allowed_first_pattern_token()
            {
                rule.pattern_add_exact(&tk);
            }
            else if tk.kind == syntax::TokenKind::BraceOpen
            {
                let tk_param_name = state.parser.expect(syntax::TokenKind::Identifier)?;
                let param_name = tk_param_name.excerpt.as_ref().unwrap().clone();

                let param_type = if let Some(_) = state.parser.maybe_expect(syntax::TokenKind::Colon)
                {
                    let tk_param_type_name = state.parser.expect(syntax::TokenKind::Identifier)?;
                    let param_type_name = tk_param_type_name.excerpt.as_ref().unwrap().clone();
                    
                    asm::PatternParameterType::RuleGroup
                    {
                        name: param_type_name
                    }
                }
                else
                {
                    asm::PatternParameterType::Unspecified
                };

                rule.pattern_add_parameter(asm::PatternParameter
                {
                    name: param_name,
                    typ: param_type,
                });
            }
            else
            {
                state.report.error_span(
                    "token is not allowed as the start of a pattern",
                    &tk.span);
                
                return Err(());
            }
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


    rule.production = expr::Expression::parse(&mut state.parser)?;

    if !rule.production.has_size()
    {
        state.report.error_span(
            "size of rule production must be known; \
            try using a bit slice like `x[hi:lo]`",
            &rule.production.span());

        return Err(());
    }

    Ok(rule)
}