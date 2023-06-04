use super::*;


#[derive(Debug)]
pub struct AstSymbol
{
    pub decl_span: diagn::Span,
    pub hierarchy_level: usize,
    pub name: Option<String>,
    pub kind: AstSymbolKind,
    
    pub item_ref: Option<util::ItemRef::<asm::Symbol>>,
}


#[derive(Debug)]
pub enum AstSymbolKind
{
    Constant(AstSymbolConstant),
    Label,
}


#[derive(Debug)]
pub struct AstSymbolConstant
{
    pub expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker)
    -> Result<AstAny, ()>
{
    let mut decl_span = diagn::Span::new_dummy();
    let mut hierarchy_level = 0;
    
    while let Some(tk_dot) = walker.maybe_expect(syntax::TokenKind::Dot)
    {
        hierarchy_level += 1;
        decl_span = decl_span.join(tk_dot.span);
    }

    let maybe_tk_name = walker.maybe_expect(syntax::TokenKind::Identifier);
    let maybe_name = maybe_tk_name.map(|tk| tk.excerpt.clone().unwrap());
    
    if let Some(tk_name) = maybe_tk_name
    {
        decl_span = decl_span.join(tk_name.span);
    }

    if walker.maybe_expect(syntax::TokenKind::Equal).is_some()
    {
        match maybe_name
        {
            None =>
            {
                report.error_span(
                    "expected identifier",
                    walker.get_span_after_prev());

                Err(())
            }
            Some(name) =>
            {
                let expr = expr::parse(report, walker)?;
                walker.expect_linebreak(report)?;
                
                Ok(AstAny::Symbol(AstSymbol {
                    decl_span,
                    hierarchy_level,
                    name: Some(name),
                    kind: AstSymbolKind::Constant(AstSymbolConstant {
                        expr,
                    }),

                    item_ref: None,
                }))
            }
        }
    }
    else
    {
        let tk_colon = walker.expect(report, syntax::TokenKind::Colon)?;
        decl_span = decl_span.join(tk_colon.span);
        
        Ok(AstAny::Symbol(AstSymbol {
            decl_span,
            hierarchy_level,
            name: maybe_name,
            kind: AstSymbolKind::Label,

            item_ref: None,
        }))
    }
}