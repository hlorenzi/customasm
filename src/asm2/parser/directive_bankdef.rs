use super::*;


#[derive(Debug)]
pub struct AstDirectiveBankdef
{
    pub header_span: diagn::Span,
    pub name_span: diagn::Span,
    pub name: String,

    pub addr_unit: Option<expr::Expr>,
    pub label_align: Option<expr::Expr>,
	pub addr_start: Option<expr::Expr>,
	pub addr_end: Option<expr::Expr>,
	pub addr_size: Option<expr::Expr>,
	pub output_offset: Option<expr::Expr>,
	pub fill: bool,
    
    pub item_ref: Option<util::ItemRef::<asm2::Bankdef>>,
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
    header_span: diagn::Span)
    -> Result<AstDirectiveBankdef, ()>
{
    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.clone().unwrap();
    let name_span = tk_name.span.clone();

    walker.expect(report, syntax::TokenKind::BraceOpen)?;

    let mut fields = fields::parse(report, walker)?;
    
    let addr_unit = fields.extract_as_optional_expr(
        report,
        "bits")?;
        
    let label_align = fields.extract_as_optional_expr(
        report,
        "labelalign")?;

    let addr_start = fields.extract_as_optional_expr(
        report,
        "addr")?;
        
    let addr_end = fields.extract_as_optional_expr(
        report,
        "addr_end")?;
        
    let addr_size = fields.extract_as_optional_expr(
        report,
        "size")?;
        
    let output_offset = fields.extract_as_optional_expr(
        report,
        "outp")?;
        
    let fill = fields.extract_as_bool(
        report,
        "fill")?;

    fields.report_remaining(report)?;

    walker.expect(report, syntax::TokenKind::BraceClose)?;
    walker.expect_linebreak(report)?;

    Ok(AstDirectiveBankdef {
        header_span,
        name_span,
        name,
        
        addr_unit,
        label_align,
        addr_start,
        addr_end,
        addr_size,
        output_offset,
        fill,

        item_ref: None,
    })
}