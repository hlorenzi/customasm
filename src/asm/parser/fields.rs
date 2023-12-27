use super::*;


pub struct AstFields
{
    span: diagn::Span,
    fields: Vec<AstField>,
}


#[derive(Debug)]
pub struct AstField
{
    pub span: diagn::Span,
    pub name: String,
    pub maybe_expr: Option<expr::Expr>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker)
    -> Result<AstFields, ()>
{
    let mut fields = AstFields {
        span: diagn::Span::new_dummy(),
        fields: Vec::new(),
    };

    while !walker.next_useful_is(0, syntax::TokenKind::BraceClose)
    {
        let deprecated_hash =
            walker.maybe_expect(syntax::TokenKind::Hash).is_some();

        let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
        let name = tk_name.excerpt.as_ref().unwrap().clone();

        if let Some(_) = fields.fields.iter().find(|f| f.name == name)
        {
            report.error_span(
                format!("duplicate field `{}`", name),
                tk_name.span);

            return Err(());
        }


        let maybe_expr = {
            if (deprecated_hash && !walker.next_linebreak().is_some()) ||
                walker.maybe_expect(syntax::TokenKind::Equal).is_some()
            {
                let expr = expr::parse(report, walker)?;
                fields.span = fields.span.join(expr.span());
                Some(expr)
            }
            else
            {
                None
            }
        };


        fields.span = fields.span.join(tk_name.span);

        fields.fields.push(AstField {
            span: tk_name.span,
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


impl AstFields
{
    pub fn extract_optional(
        &mut self,
        field_name: &str)
        -> Option<AstField>
    {
        let maybe_field = self.fields
            .iter()
            .enumerate()
            .find(|f| f.1.name == field_name);

        if let Some((index, _)) = maybe_field
        {
            let field = self.fields.remove(index);
            Some(field)
        }
        else
        {
            None
        }
    }


    pub fn extract(
        &mut self,
        report: &mut diagn::Report,
        field_name: &str)
        -> Result<AstField, ()>
    {
        match self.extract_optional(field_name)
        {
            Some(field) => Ok(field),
            None =>
            {
                report.error_span(
                    format!("missing field `{}`", field_name),
                    self.span);

                Err(())
            }
        }
    }


    pub fn extract_as_expr(
        &mut self,
        report: &mut diagn::Report,
        field_name: &str)
        -> Result<Option<expr::Expr>, ()>
    {
        let field = self.extract(report, field_name)?;
        Ok(field.maybe_expr)
    }


    pub fn extract_as_optional_expr(
        &mut self,
        _report: &mut diagn::Report,
        field_name: &str)
        -> Result<Option<expr::Expr>, ()>
    {
        let maybe_field = self.extract_optional(field_name);
        match maybe_field
        {
            Some(field) => Ok(field.maybe_expr),
            None => Ok(None),
        }
    }


    pub fn extract_as_bool(
        &mut self,
        _report: &mut diagn::Report,
        field_name: &str)
        -> Result<bool, ()>
    {
        let field = self.extract_optional(field_name);
        match field
        {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }


    pub fn report_remaining(
        &self,
        report: &mut diagn::Report)
        -> Result<(), ()>
    {
        for field in &self.fields
        {
            report.error_span(
                format!("invalid field `{}`", field.name),
                field.span);
        }

        if self.fields.len() > 0
        {
            Err(())
        }
        else
        {
            Ok(())
        }
    }
}