use super::*;


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    _header_span: diagn::Span)
    -> Result<AstSymbol, ()>
{
    let mut no_emit = false;

    if let Some(_) = walker.maybe_expect(syntax::TokenKind::ParenOpen)
    {
        let tk_attrb = walker.expect(report, syntax::TokenKind::Identifier)?;
        let attrb = tk_attrb.excerpt.as_ref().unwrap();

        match attrb.as_ref()
        {
            "noemit" => no_emit = true,
            _ =>
            {
                report.error_span(
                    format!("invalid attribute `{}`", attrb),
                    tk_attrb.span);

                return Err(());
            }
        }

        walker.expect(report, syntax::TokenKind::ParenClose)?;
    }


    let mut decl_span = diagn::Span::new_dummy();
    let mut hierarchy_level = 0;
    
    while let Some(tk_dot) = walker.maybe_expect(syntax::TokenKind::Dot)
    {
        hierarchy_level += 1;
        decl_span = decl_span.join(tk_dot.span);
    }

    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.clone().unwrap();
    decl_span = decl_span.join(tk_name.span);


    walker.expect(report, syntax::TokenKind::Equal)?;

    let expr = expr::parse(report, walker)?;
    walker.expect_linebreak(report)?;
    
    Ok(AstSymbol {
        decl_span,
        hierarchy_level,
        name,
        kind: AstSymbolKind::Constant(AstSymbolConstant {
            expr,
        }),
        no_emit,

        item_ref: None,
    })
}