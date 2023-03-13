use super::*;


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker)
    -> Result<AstAny, ()>
{
    let tk_hash = walker.expect(report, syntax::TokenKind::Hash)?;
    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let header_span = tk_hash.span.join(&tk_name.span);

    let name = tk_name.excerpt.as_ref().unwrap().to_ascii_lowercase();

    if name.chars().next() == Some('d')
    {
        if name == "d"
        {
            return Ok(AstAny::DirectiveData(
                directive_data::parse(
                    report,
                    walker,
                    None,
                    header_span)?));
        }
        else if let Ok(elem_size) = usize::from_str_radix(&name[1..], 10)
        {
            return Ok(AstAny::DirectiveData(
                directive_data::parse(
                    report,
                    walker,
                    Some(elem_size),
                    header_span)?));
        }
    }
    
    match name.as_ref()
    {
        "addr" => Ok(AstAny::DirectiveAddr(
            directive_addr::parse(report, walker, header_span)?)),
        
        "align" => Ok(AstAny::DirectiveAlign(
            directive_align::parse(report, walker, header_span)?)),
        
        "bank" => Ok(AstAny::DirectiveBank(
            directive_bank::parse(report, walker, header_span)?)),
        
        "bankdef" => Ok(AstAny::DirectiveBankdef(
            directive_bankdef::parse(report, walker, header_span)?)),
            
        "bits" => Ok(AstAny::DirectiveBits(
            directive_bits::parse(report, walker, header_span)?)),
        
        "fn" => Ok(AstAny::DirectiveFn(
            directive_fn::parse(report, walker, header_span)?)),
        
        "include" => Ok(AstAny::DirectiveInclude(
            directive_include::parse(report, walker, header_span)?)),
        
        "labelalign" => Ok(AstAny::DirectiveLabelAlign(
            directive_labelalign::parse(report, walker, header_span)?)),
        
        "noemit" => Ok(AstAny::DirectiveNoEmit(
            directive_noemit::parse(report, walker, header_span)?)),
        
        "once" => Ok(AstAny::DirectiveOnce(
            directive_once::parse(report, walker, header_span)?)),
            
        "res" => Ok(AstAny::DirectiveRes(
            directive_res::parse(report, walker, header_span)?)),
        
        "ruledef" => Ok(AstAny::DirectiveRuledef(
            directive_ruledef::parse(report, walker, false, header_span)?)),
        
        "subruledef" => Ok(AstAny::DirectiveRuledef(
            directive_ruledef::parse(report, walker, true, header_span)?)),
        
        _ =>
        {
            report.error_span(
                format!("unknown directive `{}`", name),
                &header_span);
            
            Err(())
        }
    }
}