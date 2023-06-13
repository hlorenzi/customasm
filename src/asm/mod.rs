use crate::*;


pub mod parser;
pub use parser::{
    AstAny,
    AstDirectiveAddr,
    AstDirectiveAlign,
    AstDirectiveBank,
    AstDirectiveBankdef,
    AstDirectiveBits,
    AstDirectiveData,
    AstDirectiveFn,
    AstDirectiveInclude,
    AstDirectiveLabelAlign,
    AstDirectiveNoEmit,
    AstDirectiveOnce,
    AstDirectiveRes,
    AstDirectiveRuledef,
    AstField,
    AstFields,
    AstFnParameter,
    AstInstruction,
    AstSymbol,
    AstSymbolKind,
    AstSymbolConstant,
    AstRule,
    AstRuleParameter,
    AstRuleParameterType,
    AstRulePatternPart,
    AstTopLevel,
};

pub mod decls;
pub use decls::{
    ItemDecls,
};

pub mod defs;
pub use defs::{
    ItemDefs,
    Bankdef,
    Ruledef,
    RuledefMap,
    RuledefMapEntry,
    Rule,
    RuleParameter,
    RuleParameterType,
    RulePattern,
    RulePatternPart,
    Symbol,
    Function,
    FunctionParameter,
    Instruction,
    DataElement,
    ResDirective,
    AlignDirective,
    AddrDirective,
};

pub mod matcher;
pub use matcher::{
    InstructionMatches,
    InstructionMatch,
    InstructionMatchResolution,
    InstructionArgument,
    InstructionArgumentKind,
};

pub mod resolver;
pub use resolver::{
    ResolutionState,
    ResolveIterator,
    ResolverContext,
    ResolverNode,
};

pub mod output;


pub struct AssemblyResult
{
    pub error: bool,
    pub ast: Option<asm::AstTopLevel>,
    pub decls: Option<asm::ItemDecls>,
    pub defs: Option<asm::ItemDefs>,
    pub output: Option<util::BitVec>,
    pub iterations_taken: Option<usize>,
}


pub struct AssemblyOptions
{
    pub max_iterations: usize,
    pub debug_iterations: bool,
    pub optimize_statically_known: bool,
    pub optimize_instruction_matching: bool,
}


impl AssemblyResult
{
    pub fn new() -> AssemblyResult
    {
        AssemblyResult {
            error: false,
            ast: None,
            decls: None,
            defs: None,
            output: None,
            iterations_taken: None,
        }
    }
}


impl AssemblyOptions
{
    pub fn new() -> AssemblyOptions
    {
        AssemblyOptions {
            max_iterations: 10,
            debug_iterations: false,
            optimize_statically_known: true,
            optimize_instruction_matching: true,
        }
    }
}


pub fn assemble<S>(
    report: &mut diagn::Report,
    opts: &AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    root_filenames: &[S])
    -> AssemblyResult
    where S: std::borrow::Borrow<str>
{
    let mut assembly = AssemblyResult::new();

    let mut run = || -> Result<(), ()>
    {
        assembly.ast = Some(parser::parse_many_and_resolve_includes(
            report,
            fileserver,
            root_filenames)?);

        assembly.decls = Some(decls::init(report)?);

        assembly.defs = Some(defs::init());

        let mut prev_resolved_constants_count = 0;

        loop
        {
            decls::collect(
                report,
                assembly.ast.as_mut().unwrap(),
                assembly.decls.as_mut().unwrap())?;

            defs::define_symbols(
                report,
                opts,
                assembly.ast.as_mut().unwrap(),
                assembly.decls.as_ref().unwrap(),
                assembly.defs.as_mut().unwrap())?;
                
            let resolved_constants_count = resolver::resolve_constants_simple(
                report,
                opts,
                fileserver,
                assembly.ast.as_ref().unwrap(),
                assembly.decls.as_ref().unwrap(),
                assembly.defs.as_mut().unwrap())?;
    
            let resolved_ifs_count = resolver::resolve_ifs(
                report,
                opts,
                fileserver,
                assembly.ast.as_mut().unwrap(),
                assembly.decls.as_ref().unwrap(),
                assembly.defs.as_mut().unwrap())?;

            if resolved_constants_count == prev_resolved_constants_count &&
                resolved_ifs_count == 0
            {
                break;
            }

            prev_resolved_constants_count = resolved_constants_count;
        }

        resolver::check_leftover_ifs(
            report,
            assembly.ast.as_ref().unwrap(),
            assembly.decls.as_ref().unwrap(),
            assembly.defs.as_ref().unwrap())?;
            
        defs::define_remaining(
            report,
            opts,
            assembly.ast.as_mut().unwrap(),
            assembly.defs.as_mut().unwrap(),
            assembly.decls.as_mut().unwrap())?;

        matcher::match_all(
            report,
            opts,
            assembly.ast.as_ref().unwrap(),
            assembly.decls.as_ref().unwrap(),
            assembly.defs.as_mut().unwrap())?;

        assembly.iterations_taken = Some(resolver::resolve_iteratively(
            report,
            opts,
            fileserver,
            assembly.ast.as_ref().unwrap(),
            assembly.decls.as_ref().unwrap(),
            assembly.defs.as_mut().unwrap(),
            opts.max_iterations)?);

        output::check_bank_overlap(
            report,
            assembly.decls.as_ref().unwrap(),
            assembly.defs.as_mut().unwrap())?;

        assembly.output = Some(output::build_output(
            report,
            assembly.ast.as_ref().unwrap(),
            assembly.decls.as_ref().unwrap(),
            assembly.defs.as_ref().unwrap())?);

        Ok(())
    };
    
    match run()
    {
        Ok(()) => {}
        Err(()) =>
        {
            assembly.error = true;
            assert!(report.has_errors());
        }
    }

    assembly
}