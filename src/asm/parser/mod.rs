use crate::*;


mod directive;

mod directive_addr;
pub use directive_addr::AstDirectiveAddr;

mod directive_align;
pub use directive_align::AstDirectiveAlign;

mod directive_bank;
pub use directive_bank::AstDirectiveBank;

mod directive_bankdef;
pub use directive_bankdef::AstDirectiveBankdef;

mod directive_bits;
pub use directive_bits::AstDirectiveBits;

mod directive_const;

mod directive_data;
pub use directive_data::AstDirectiveData;

mod directive_fn;
pub use directive_fn::{
    AstDirectiveFn,
    AstFnParameter,
};

mod directive_if;
pub use directive_if::AstDirectiveIf;

mod directive_include;
pub use directive_include::AstDirectiveInclude;

mod directive_labelalign;
pub use directive_labelalign::AstDirectiveLabelAlign;

mod directive_noemit;
pub use directive_noemit::AstDirectiveNoEmit;

mod directive_once;
pub use directive_once::AstDirectiveOnce;

mod directive_res;
pub use directive_res::AstDirectiveRes;

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
pub use instruction::AstInstruction;

mod symbol;
pub use symbol::{
    AstSymbol,
    AstSymbolKind,
    AstSymbolConstant,
};


#[derive(Clone, Debug)]
pub enum AstAny
{
    DirectiveAddr(AstDirectiveAddr),
    DirectiveAlign(AstDirectiveAlign),
    DirectiveBank(AstDirectiveBank),
    DirectiveBankdef(AstDirectiveBankdef),
    DirectiveBits(AstDirectiveBits),
    DirectiveData(AstDirectiveData),
    DirectiveFn(AstDirectiveFn),
    DirectiveIf(AstDirectiveIf),
    DirectiveInclude(AstDirectiveInclude),
    DirectiveLabelAlign(AstDirectiveLabelAlign),
    DirectiveNoEmit(AstDirectiveNoEmit),
    DirectiveOnce(AstDirectiveOnce),
    DirectiveRes(AstDirectiveRes),
    DirectiveRuledef(AstDirectiveRuledef),
    Instruction(AstInstruction),
    Symbol(AstSymbol),
}


#[derive(Clone, Debug)]
pub struct AstTopLevel
{
    pub nodes: Vec<AstAny>,
}


pub fn parse_many_and_resolve_includes<S>(
    report: &mut diagn::Report,
    fileserver: &mut dyn util::FileServer,
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
            None,
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
    span: Option<diagn::Span>,
    fileserver: &mut dyn util::FileServer,
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

    let file_handle = fileserver.get_handle(
        report,
        span,
        root_filename.borrow())?;

    let src = fileserver.get_str(
        report,
        span,
        file_handle)?;

    let mut walker = syntax::Walker::new(
        &src,
        file_handle,
        0);

    let mut root_ast = parse(report, &mut walker)?;

    // Check presence of an #once directive
    if root_ast.nodes.iter().any(|n| matches!(n, AstAny::DirectiveOnce(_)))
    {
        once_filenames.insert(root_filename.borrow().to_owned());
    }

    // Recursively find and replace our `#include` AST nodes
    // with the full ASTs of the included files
    let mut node_index = 0;
    while node_index < root_ast.nodes.len()
    {
        let node = &root_ast.nodes[node_index];

        if let AstAny::DirectiveInclude(ast_include) = node
        {
            let included_filename = util::filename_navigate(
                report,
                ast_include.filename_span,
                root_filename.borrow(),
                &ast_include.filename)?;


            if seen_filenames.contains(&included_filename)
            {
                report.error_span(
                    "recursive file inclusion",
                    ast_include.filename_span);

                return Err(());
            }

    
            seen_filenames.push(included_filename.clone());

            let inner_ast = parse_and_resolve_includes(
                report,
                Some(ast_include.filename_span),
                fileserver,
                included_filename.as_ref(),
                seen_filenames,
                once_filenames)?;

            let inner_ast_len = inner_ast.nodes.len();

            root_ast.nodes.splice(
                node_index..(node_index + 1),
                inner_ast.nodes);

            // Skip over the included AST since it already
            // had its own `#include` nodes handled and replaced
            node_index += inner_ast_len;
            
            seen_filenames.pop();
        }
        else
        {
            node_index += 1;
        }
    }

    Ok(root_ast)
}


pub fn parse(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker)
    -> Result<AstTopLevel, ()>
{
    let mut nodes = Vec::new();
    
    while !walker.is_over()
    {
        if let Some(node) = parse_line(report, walker)?
        {
            nodes.push(node);
        }
    }

    Ok(AstTopLevel {
        nodes
    })
}


pub fn parse_nested_toplevel(
    report: &mut diagn::Report,
    walker: &mut syntax::Walker)
    -> Result<AstTopLevel, ()>
{
    let mut nodes = Vec::new();
    
    while !walker.is_over() &&
        !walker.next_useful_is(0, syntax::TokenKind::BraceClose)
    {
        if let Some(node) = parse_line(report, walker)?
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
    walker: &mut syntax::Walker)
    -> Result<Option<AstAny>, ()>
{
    // Directives (starting with a hash sign)
    if walker.next_useful_is(0, syntax::TokenKind::Hash)
    {
        Ok(Some(directive::parse(report, walker)?))
    }

    // Global labels (identifiers followed by colons)
    else if walker.next_useful_is(0, syntax::TokenKind::Identifier) &&
        walker.next_useful_is(1, syntax::TokenKind::Colon)
    {
        Ok(Some(symbol::parse(report, walker)?))
    }

    // Global constants (identifiers followed by equal signs)
    else if walker.next_useful_is(0, syntax::TokenKind::Identifier) &&
        walker.next_useful_is(1, syntax::TokenKind::Equal)
    {
        Ok(Some(symbol::parse(report, walker)?))
    }

    // Local labels or constants (starting with a dot)
    else if walker.next_useful_is(0, syntax::TokenKind::Dot)
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
    pub fn span(&self) -> diagn::Span
    {
        match self
        {
            AstAny::DirectiveAddr(node) => node.header_span,
            AstAny::DirectiveAlign(node) => node.header_span,
            AstAny::DirectiveBank(node) => node.header_span,
            AstAny::DirectiveBankdef(node) => node.header_span,
            AstAny::DirectiveBits(node) => node.header_span,
            AstAny::DirectiveData(node) => node.header_span,
            AstAny::DirectiveFn(node) => node.header_span,
            AstAny::DirectiveIf(node) => node.header_span,
            AstAny::DirectiveInclude(node) => node.header_span,
            AstAny::DirectiveLabelAlign(node) => node.header_span,
            AstAny::DirectiveNoEmit(node) => node.header_span,
            AstAny::DirectiveOnce(node) => node.header_span,
            AstAny::DirectiveRes(node) => node.header_span,
            AstAny::DirectiveRuledef(node) => node.header_span,
            AstAny::Instruction(node) => node.span,
            AstAny::Symbol(node) => node.decl_span,
        }
    }
}