use crate::*;


mod directive;

mod directive_addr;
pub use directive_addr::{
    AstDirectiveAddr,
};

mod directive_align;
pub use directive_align::{
    AstDirectiveAlign,
};

mod directive_bank;
pub use directive_bank::{
    AstDirectiveBank,
};

mod directive_bankdef;
pub use directive_bankdef::{
    AstDirectiveBankdef,
};

mod directive_bits;
pub use directive_bits::{
    AstDirectiveBits,
};

mod directive_data;
pub use directive_data::{
    AstDirectiveData,
};

mod directive_fn;
pub use directive_fn::{
    AstDirectiveFn,
    AstFnParameter,
};

mod directive_include;
pub use directive_include::{
    AstDirectiveInclude,
};

mod directive_labelalign;
pub use directive_labelalign::{
    AstDirectiveLabelAlign,
};

mod directive_noemit;
pub use directive_noemit::{
    AstDirectiveNoEmit,
};

mod directive_once;
pub use directive_once::{
    AstDirectiveOnce,
};

mod directive_res;
pub use directive_res::{
    AstDirectiveRes,
};

mod directive_ruledef;
pub use directive_ruledef::{
    AstDirectiveRuledef,
    AstRule,
    AstRulePatternPart,
    AstRuleParameter,
    AstRuleParameterType,
};

mod fields;
pub use fields::{
    AstFields,
    AstField,
};

mod instruction;
pub use instruction::{
    AstInstruction,
};

mod symbol;
pub use symbol::{
    AstSymbol,
    AstSymbolKind,
    AstSymbolConstant,
};


#[derive(Debug)]
pub enum AstAny
{
    DirectiveAddr(AstDirectiveAddr),
    DirectiveAlign(AstDirectiveAlign),
    DirectiveBank(AstDirectiveBank),
    DirectiveBankdef(AstDirectiveBankdef),
    DirectiveBits(AstDirectiveBits),
    DirectiveData(AstDirectiveData),
    DirectiveFn(AstDirectiveFn),
    DirectiveInclude(AstDirectiveInclude),
    DirectiveLabelAlign(AstDirectiveLabelAlign),
    DirectiveNoEmit(AstDirectiveNoEmit),
    DirectiveOnce(AstDirectiveOnce),
    DirectiveRes(AstDirectiveRes),
    DirectiveRuledef(AstDirectiveRuledef),
    Instruction(AstInstruction),
    Symbol(AstSymbol),
}


#[derive(Debug)]
pub struct AstTopLevel
{
    pub nodes: Vec<AstAny>,
}


pub fn parse_many_and_resolve_includes<S>(
    report: &mut diagn::Report,
    fileserver: &dyn util::FileServer,
    root_filenames: &[S])
    -> Result<AstTopLevel, ()>
    where S: std::borrow::Borrow<str>
{
    let mut result = AstTopLevel {
        nodes: Vec::new(),
    };

    let mut once_filenames = std::collections::HashSet::new();

    for file in root_filenames
    {
        let ast = parse_and_resolve_includes(
            report,
            fileserver,
            file.borrow(),
            &mut Vec::new(),
            &mut once_filenames)?;

        result.nodes.extend(ast.nodes);
    }

    Ok(result)
}


pub fn parse_and_resolve_includes<S>(
    report: &mut diagn::Report,
    fileserver: &dyn util::FileServer,
    root_filename: S,
    seen_filenames: &mut Vec<String>,
    once_filenames: &mut std::collections::HashSet<String>)
    -> Result<AstTopLevel, ()>
    where S: std::borrow::Borrow<str>
{
    if once_filenames.contains(root_filename.borrow())
    {
        return Ok(AstTopLevel {
            nodes: Vec::new(),
        });
    }

    let chars = fileserver.get_chars(
        report,
        None,
        root_filename.borrow())?;

    let tokens = syntax::tokenize(
        report,
        root_filename.borrow(),
        &chars)?;

    let mut root_ast = parse(report, &tokens)?;

    // Check presence of an #once directive
    if root_ast.nodes.iter().any(|n| matches!(n, AstAny::DirectiveOnce(_)))
    {
        once_filenames.insert(root_filename.borrow().to_owned());
    }

    for node_index in (0..root_ast.nodes.len()).rev()
    {
        let node = &root_ast.nodes[node_index];

        if let AstAny::DirectiveInclude(dir_include) = node
        {
            let included_filename = util::filename_navigate(
                report,
                &dir_include.filename_span,
                root_filename.borrow(),
                &dir_include.filename)?;


            if seen_filenames.contains(&included_filename)
            {
                report.error_span(
                    "recursive file inclusion",
                    &dir_include.filename_span);

                return Err(());
            }

    
            seen_filenames.push(included_filename.clone());

            let inner_ast = parse_and_resolve_includes(
                report,
                fileserver,
                included_filename.as_ref(),
                seen_filenames,
                once_filenames)?;

            // Replace the `#include` node with
            // the actual included file's AST
            root_ast.nodes.splice(
                node_index..(node_index + 1),
                inner_ast.nodes);
            
            seen_filenames.pop();
        }
    }

    Ok(root_ast)
}


pub fn parse(
    report: &mut diagn::Report,
    tokens: &[syntax::Token])
    -> Result<AstTopLevel, ()>
{
    let mut walker = syntax::TokenWalker::new(tokens);

    let mut nodes = Vec::new();
    
    while !walker.is_over()
    {
        if let Some(node) = parse_line(report, &mut walker)?
        {
            nodes.push(node);
        }
    }

    Ok(AstTopLevel {
        nodes
    })
}


fn parse_line(
    report: &mut diagn::Report,
    walker: &mut syntax::TokenWalker)
    -> Result<Option<AstAny>, ()>
{
    // Directives (starting with a hash sign)
    if walker.next_is(0, syntax::TokenKind::Hash)
    {
        Ok(Some(directive::parse(report, walker)?))
    }

    // Global labels (identifiers followed by colons)
    else if walker.next_is(0, syntax::TokenKind::Identifier) &&
        walker.next_is(1, syntax::TokenKind::Colon)
    {
        Ok(Some(symbol::parse(report, walker)?))
    }

    // Global constants (identifiers followed by equal signs)
    else if walker.next_is(0, syntax::TokenKind::Identifier) &&
        walker.next_is(1, syntax::TokenKind::Equal)
    {
        Ok(Some(symbol::parse(report, walker)?))
    }

    // Local labels or constants (starting with a dot)
    else if walker.next_is(0, syntax::TokenKind::Dot)
    {
        Ok(Some(symbol::parse(report, walker)?))
    }

    // Empty lines
    else if walker.maybe_expect_linebreak().is_some()
    {
        Ok(None)
    }

    // Everything else is regarded as an instruction
    else
    {
        Ok(Some(AstAny::Instruction(
            instruction::parse(report, walker)?)))
    }
}


impl AstAny
{
    pub fn span(&self) -> &diagn::Span
    {
        match self
        {
            AstAny::DirectiveAddr(node) => &node.header_span,
            AstAny::DirectiveAlign(node) => &node.header_span,
            AstAny::DirectiveBank(node) => &node.header_span,
            AstAny::DirectiveBankdef(node) => &node.header_span,
            AstAny::DirectiveBits(node) => &node.header_span,
            AstAny::DirectiveData(node) => &node.header_span,
            AstAny::DirectiveFn(node) => &node.header_span,
            AstAny::DirectiveInclude(node) => &node.header_span,
            AstAny::DirectiveLabelAlign(node) => &node.header_span,
            AstAny::DirectiveNoEmit(node) => &node.header_span,
            AstAny::DirectiveOnce(node) => &node.header_span,
            AstAny::DirectiveRes(node) => &node.header_span,
            AstAny::DirectiveRuledef(node) => &node.header_span,
            AstAny::Instruction(node) => &node.span,
            AstAny::Symbol(node) => &node.decl_span,
        }
    }
}