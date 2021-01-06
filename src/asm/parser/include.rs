use crate::*;


pub fn parse_directive_include(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let tk_filename = state.parser.expect(syntax::TokenKind::String)?;
    let filename = syntax::excerpt_as_string_contents(
        state.report.clone(),
        tk_filename.excerpt.as_ref().unwrap(),
        &tk_filename.span)?;

    let new_filename = util::filename_navigate(
        state.report.clone(),
        &state.filename,
        &filename,
        &tk_filename.span)?;

    if state.parsed_filenames.contains(&new_filename)
    {
        state.report.error_span("recursive include", &tk_filename.span);
        return Err(());
    }

    asm::parser::parse_file(
        state.report.clone(),
        state.asm_state,
        state.fileserver,
        new_filename,
        Some(&tk_filename.span),
        state.parsed_filenames)
}