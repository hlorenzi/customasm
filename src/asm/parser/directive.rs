use crate::*;

pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker,
) -> Result<asm::AstAny, ()>
{
    let tk_hash = walker.expect(report, syntax::TokenKind::Hash)?;
    let tk_name = walker.expect(report, syntax::TokenKind::Identifier)?;
    let header_span = tk_hash.span.join(tk_name.span);

    let name = tk_name.excerpt.as_ref().unwrap().to_ascii_lowercase();

    if name.chars().next() == Some('d')
    {
        if name == "d"
        {
            return Ok(asm::AstAny::DirectiveData(
                asm::parser::directive_data::parse(report, walker, None, header_span)?,
            ));
        }
        else if let Ok(elem_size) = usize::from_str_radix(&name[1..], 10)
        {
            return Ok(asm::AstAny::DirectiveData(
                asm::parser::directive_data::parse(report, walker, Some(elem_size), header_span)?,
            ));
        }
    }

    match name.as_ref()
    {
        "addr" => Ok(asm::AstAny::DirectiveAddr(
            asm::parser::directive_addr::parse(report, walker, header_span)?,
        )),

        "align" => Ok(asm::AstAny::DirectiveAlign(
            asm::parser::directive_align::parse(report, walker, header_span)?,
        )),

        "bank" => Ok(asm::AstAny::DirectiveBank(
            asm::parser::directive_bank::parse(report, walker, header_span)?,
        )),

        "bankdef" => Ok(asm::AstAny::DirectiveBankdef(
            asm::parser::directive_bankdef::parse(report, walker, header_span)?,
        )),

        "bits" => Ok(asm::AstAny::DirectiveBits(
            asm::parser::directive_bits::parse(report, walker, header_span)?,
        )),

        "const" => Ok(asm::AstAny::Symbol(asm::parser::directive_const::parse(
            report,
            walker,
            header_span,
        )?)),

        "fn" => Ok(asm::AstAny::DirectiveFn(asm::parser::directive_fn::parse(
            report,
            walker,
            header_span,
        )?)),

        "if" => Ok(asm::AstAny::DirectiveIf(asm::parser::directive_if::parse(
            report,
            walker,
            header_span,
        )?)),

        "include" => Ok(asm::AstAny::DirectiveInclude(
            asm::parser::directive_include::parse(report, walker, header_span)?,
        )),

        "labelalign" => Ok(asm::AstAny::DirectiveLabelAlign(
            asm::parser::directive_labelalign::parse(report, walker, header_span)?,
        )),

        "noemit" => Ok(asm::AstAny::DirectiveNoEmit(
            asm::parser::directive_noemit::parse(report, walker, header_span)?,
        )),

        "once" => Ok(asm::AstAny::DirectiveOnce(
            asm::parser::directive_once::parse(report, walker, header_span)?,
        )),

        "res" => Ok(asm::AstAny::DirectiveRes(
            asm::parser::directive_res::parse(report, walker, header_span)?,
        )),

        "ruledef" => Ok(asm::AstAny::DirectiveRuledef(
            asm::parser::directive_ruledef::parse(report, walker, false, header_span)?,
        )),

        "subruledef" => Ok(asm::AstAny::DirectiveRuledef(
            asm::parser::directive_ruledef::parse(report, walker, true, header_span)?,
        )),

        _ =>
        {
            report.error_span(format!("unknown directive `{}`", name), header_span);

            Err(())
        }
    }
}
