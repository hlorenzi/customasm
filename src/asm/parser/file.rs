use crate::*;


pub fn parse_file<TFilename: Into<String>>(
    report: diagn::RcReport,
    asm_state: &mut asm::State,
    fileserver: &dyn util::FileServer,
    filename: TFilename)
    -> Result<(), ()>
{
    let filename = filename.into();
    let chars = fileserver.get_chars(report.clone(), &filename, None)?;
    let tokens = syntax::tokenize(report.clone(), &filename, &chars)?;
    let parser = syntax::Parser::new(Some(report.clone()), &tokens);
    
    let mut state = asm::parser::State
    {
        report,
        asm_state,
        fileserver,
        filename,
        
        parser,
    };

    //println!("{:#?}", state.parser.tokens.iter().map(|t| t.kind).collect::<Vec<_>>());

    while !state.parser.is_over()
    {
        parse_line(&mut state)?;
    }
		
	Ok(())
}


pub fn parse_line(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    if state.parser.next_is(0, syntax::TokenKind::Hash)
    {
        parse_directive(state)?;
    }
    else if state.parser.maybe_expect_linebreak().is_some()
    {
        return Ok(());
    }
    else
    {
        asm::parser::parse_rule_invokation(state)?;
    }

    Ok(())
}


pub fn parse_directive(state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_hash = state.parser.expect(syntax::TokenKind::Hash)?;
    let tk_directive = state.parser.expect(syntax::TokenKind::Identifier)?;

    let directive = tk_directive.excerpt.as_ref().unwrap().to_ascii_lowercase();

    match directive.as_ref()
    {
        "rulesdef" => asm::parser::parse_directive_rulesdef(state)?,
        "use" => asm::parser::parse_directive_use(state)?,
        _ =>
        {
            state.report.error_span("unknown directive", &tk_hash.span.join(&tk_directive.span));
            return Err(());
        }
    }

    state.parser.expect_linebreak()
}