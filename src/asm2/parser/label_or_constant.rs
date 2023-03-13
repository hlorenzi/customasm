use super::*;


#[derive(Debug)]
pub struct AstLabel
{
    pub decl_span: diagn::Span,
    pub depth: usize,
    pub name: String,
}


#[derive(Debug)]
pub struct AstConstant
{
    pub decl_span: diagn::Span,
    pub depth: usize,
    pub name: String,
    pub expr: expr::Expr,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker)
    -> Result<AstAny, ()>
{
    let mut decl_span = diagn::Span::new_dummy();
    let mut depth = 0;
    
    while let Some(tk_dot) = walker.maybe_expect(syntax::TokenKind::Dot)
    {
        depth += 1;
        decl_span = decl_span.join(&tk_dot.span);
    }

    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.clone().unwrap();
    decl_span = decl_span.join(&tk_name.span);


    if walker.maybe_expect(syntax::TokenKind::Equal).is_some()
    {
        let expr = expr::parse(report, walker)?;
        walker.expect_linebreak(report)?;
        
        Ok(AstAny::Constant(AstConstant {
            decl_span,
            depth,
            name,
            expr,
        }))
    }
    else
    {
        let tk_colon = walker.expect(report, syntax::TokenKind::Colon)?;
        decl_span = decl_span.join(&tk_colon.span);
        
        Ok(AstAny::Label(AstLabel {
            decl_span,
            depth,
            name,
        }))
    }
}