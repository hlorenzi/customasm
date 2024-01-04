use super::*;


#[derive(Clone, Debug)]
pub struct AstSymbol
{
    pub decl_span: diagn::Span,
    pub hierarchy_level: usize,
    pub name: String,
    pub kind: AstSymbolKind,
    pub no_emit: bool,
    
    pub item_ref: Option<util::ItemRef::<asm::Symbol>>,
}


#[derive(Clone, Debug)]
pub enum AstSymbolKind
{
    Constant(AstSymbolConstant),
    Label,
}


#[derive(Clone, Debug)]
pub struct AstSymbolConstant
{
    pub expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker)
    -> Result<AstAny, ()>
{
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


    if walker.maybe_expect(syntax::TokenKind::Equal).is_some()
    {
        let expr = expr::parse(report, walker)?;
        walker.expect_linebreak(report)?;
        
        Ok(AstAny::Symbol(AstSymbol {
            decl_span,
            hierarchy_level,
            name,
            kind: AstSymbolKind::Constant(AstSymbolConstant {
                expr,
            }),
            no_emit: false,

            item_ref: None,
        }))
    }
    else
    {
        let tk_colon = walker.expect(report, syntax::TokenKind::Colon)?;
        decl_span = decl_span.join(tk_colon.span);
        
        Ok(AstAny::Symbol(AstSymbol {
            decl_span,
            hierarchy_level,
            name,
            kind: AstSymbolKind::Label,
            no_emit: false,

            item_ref: None,
        }))
    }
}