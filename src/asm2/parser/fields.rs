use super::*;


pub type AstFields = Vec<AstField>;


#[derive(Debug)]
pub struct AstField
{
    pub span: diagn::Span,
    pub name: String,
    pub maybe_expr: Option<expr::Expr>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker)
    -> Result<AstFields, ()>
{
    let mut fields = AstFields::new();

    while !walker.next_is(0, syntax::TokenKind::BraceClose)
    {
        let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
        let name = tk_name.excerpt.as_ref().unwrap().clone();

        let maybe_expr = {
            if walker.maybe_expect(syntax::TokenKind::Equal).is_some()
            {
                Some(expr::parse(report, walker)?)
            }
            else
            {
                None
            }
        };

        fields.push(AstField {
            span: tk_name.span.clone(),
            name,
            maybe_expr,
        });
        
        if !walker.maybe_expect(syntax::TokenKind::Comma).is_some() &&
            !walker.maybe_expect_linebreak().is_some()
        {
            break;
        }
    }

    Ok(fields)
}


pub fn validate_names(
    report: &mut diagn::Report,
    fields: &AstFields,
    valid_names: &[&str])
    -> Result<(), ()>
{
    let mut had_error = false;

    for field in fields
    {
        if !valid_names.contains(&field.name.as_ref())
        {
            report.error_span(
                format!("invalid field `{}`", field.name),
                &field.span);

            had_error = true;
        }
    }

    if had_error
    {
        Err(())
    }
    else
    {
        Ok(())
    }
}