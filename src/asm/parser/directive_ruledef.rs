use crate::*;


#[derive(Clone, Debug)]
pub struct AstDirectiveRuledef
{
    pub header_span: diagn::Span,
    pub name_span: diagn::Span,
    pub is_subruledef: bool,
    pub name: Option<String>,
    pub rules: Vec<AstRule>,

    pub item_ref: Option<util::ItemRef::<asm::Ruledef>>,
}


#[derive(Clone, Debug)]
pub struct AstRule
{
    pub pattern_span: diagn::Span,
    pub pattern: Vec<AstRulePatternPart>,
    pub expr: expr::Expr,
}


#[derive(Clone, Debug)]
pub enum AstRulePatternPart
{
    Whitespace,
    Exact(char),
    Parameter(AstRuleParameter),
}


#[derive(Clone, Debug)]
pub struct AstRuleParameter
{
    pub name_span: diagn::Span,
    pub type_span: diagn::Span,
    pub name: String,
    pub typ: AstRuleParameterType,
}


#[derive(Clone, Debug)]
pub enum AstRuleParameterType
{
    Unspecified,
    Ruledef(String),
    Unsigned(usize),
    Signed(usize),
    Integer(usize),
}


pub fn parse(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    walker: &mut syntax::Walker,
    is_subruledef: bool,
    header_span: diagn::Span)
    -> Result<AstDirectiveRuledef, ()>
{
    let tk_name = walker.maybe_expect(syntax::TokenKind::Identifier);
    let name = tk_name.clone().map(|tk| walker.get_span_excerpt(tk.span).to_string());
    let name_span = tk_name
        .map(|tk| tk.span)
        .unwrap_or_else(|| header_span);

    walker.expect(report, syntax::TokenKind::BraceOpen)?;

    let mut rules = Vec::new();

    while !walker.next_useful_is(0, syntax::TokenKind::BraceClose)
    {
        let rule = parse_rule(
            report,
            opts,
            walker,
            is_subruledef)?;
        
        walker.expect_linebreak(report)?;

        rules.push(rule);
    }

    walker.expect(report, syntax::TokenKind::BraceClose)?;
    walker.expect_linebreak(report)?;

    Ok(AstDirectiveRuledef {
        header_span,
        name_span,
        is_subruledef,
        name,
        rules,

        item_ref: None,
    })
}


fn parse_rule(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    walker: &mut syntax::Walker,
    is_subruledef: bool)
    -> Result<AstRule, ()>
{
    let mut pattern_span = diagn::Span::new_dummy();
    let mut pattern = Vec::new();
    let mut has_used_empty_specifier = false;


    // Discard leading whitespace/indentation
    walker.skip_ignorable();


    while !walker.is_over() &&
        !walker.next_useful_is(0, syntax::TokenKind::HeavyArrowRight)
    {
        let tk = walker.next_token();
        walker.advance_to_token_end(&tk);
        
        pattern_span = pattern_span.join(tk.span);


        if tk.kind == syntax::TokenKind::BraceOpen
        {
            if pattern.len() == 0 &&
                is_subruledef &&
                walker.maybe_expect(syntax::TokenKind::BraceClose).is_some()
            {
                has_used_empty_specifier = true;
                break;
            }
            else
            {
                let param = parse_rule_parameter(report, opts, walker)?;
                pattern.push(AstRulePatternPart::Parameter(param));

                let tk_close = walker.expect(report, syntax::TokenKind::BraceClose)?;
                pattern_span = pattern_span.join(tk_close.span);
            }
        }
        
        else if tk.kind.is_allowed_pattern_token()
        {
            for c in walker.get_span_excerpt(tk.span).chars()
            {
                pattern.push(AstRulePatternPart::Exact(c.to_ascii_lowercase()));
            }
        }

        else if tk.kind == syntax::TokenKind::Whitespace
        {
            pattern.push(AstRulePatternPart::Whitespace);
        }
        
        else
        {
            report.error_span(
                "invalid pattern token",
                tk.span);

            return Err(());
        }
    }


    let tk_heavy_arrow = walker.expect(report, syntax::TokenKind::HeavyArrowRight)?;

    if pattern.len() == 0 && !has_used_empty_specifier
    {
        report.error_span(
            "expected pattern",
            tk_heavy_arrow.span.before());
        
        return Err(());
    }


    let expr = expr::parse(report, walker)?;

    Ok(AstRule {
        pattern_span,
        pattern,
        expr,
    })
}


fn parse_rule_parameter(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    walker: &mut syntax::Walker)
    -> Result<AstRuleParameter, ()>
{
    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let name = walker.get_span_excerpt(tk_name.span).to_string();
    let name_span = tk_name.span;

    asm::check_reserved_name(
        report,
        name_span,
        opts,
        &name)?;

    let (typ, type_span) = {
        if walker.maybe_expect(syntax::TokenKind::Colon).is_some()
        {
            let tk_typename = walker.expect(report, syntax::TokenKind::Identifier)?;
            let typename = walker.get_span_excerpt(tk_typename.span);
            let typ = interpret_typename(typename);
            (typ, tk_typename.span)
        }
        else
        {
            (AstRuleParameterType::Unspecified, diagn::Span::new_dummy())
        }
    };

    Ok(AstRuleParameter {
        name_span,
        type_span,
        name,
        typ,
    })
}


fn interpret_typename(
    typename: &str)
    -> AstRuleParameterType
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
                Some('u') => return AstRuleParameterType::Unsigned(size),
                Some('s') => return AstRuleParameterType::Signed(size),
                Some('i') => return AstRuleParameterType::Integer(size),
                _ => unreachable!()
            }
        }
    }

    AstRuleParameterType::Ruledef(typename.to_string())
}