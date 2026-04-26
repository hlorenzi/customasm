use crate::*;


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker,
    _header_span: diagn::Span)
    -> Result<asm::AstSymbol, ()>
{
    let mut no_emit = false;
    let mut is_extern = false;

    if let Some(_) = walker.maybe_expect(syntax::TokenKind::ParenOpen)
    {
        let tk_attrb = walker.expect(report, syntax::TokenKind::Identifier)?;
        let attrb = walker.get_span_excerpt(tk_attrb.span);

        match attrb.as_ref()
        {
            "noemit" => no_emit = true,
            "extern" => is_extern = true,
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
    let name = walker.get_span_excerpt(tk_name.span).to_string();
    decl_span = decl_span.join(tk_name.span);

    let expr = {
        if !is_extern
        {
            walker.expect(report, syntax::TokenKind::Equal)?;

            expr::parse(report, walker)?
        }
        else
        {
            expr::Expr::Literal(decl_span, expr::Value::make_unknown())
        }
    };

    walker.expect_linebreak(report)?;
    
    Ok(asm::AstSymbol {
        decl_span,
        hierarchy_level,
        name,
        kind: asm::AstSymbolKind::Constant(asm::AstSymbolConstant {
            expr,
        }),
        no_emit,
        is_extern,

        item_ref: None,
    })
}