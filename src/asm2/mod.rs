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
    pub ast: Option<asm2::AstTopLevel>,
    pub decls: Option<asm2::ItemDecls>,
    pub defs: Option<asm2::ItemDefs>,
    pub output: Option<util::BitVec>,
    pub iterations_taken: Option<usize>,
}


pub struct AssemblyOptions
{
    pub debug_iterations: bool,
}


pub fn assemble<S>(
    report: &mut diagn::Report,
    opts: &AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    root_filenames: &[S])
    -> AssemblyResult
    where S: std::borrow::Borrow<str>
{
    let mut assembly = AssemblyResult {
        error: false,
        ast: None,
        decls: None,
        defs: None,
        output: None,
        iterations_taken: None,
    };

    let mut run = || -> Result<(), ()>
    {
        assembly.ast = Some(parser::parse_many_and_resolve_includes(
            report,
            fileserver,
            root_filenames)?);

        assembly.decls = Some(decls::collect(
            report,
            assembly.ast.as_mut().unwrap())?);
            
        assembly.defs = Some(defs::define(
            report,
            assembly.ast.as_mut().unwrap(),
            assembly.decls.as_mut().unwrap())?);
            
        resolver::resolve_constants(
            report,
            opts,
            fileserver,
            assembly.ast.as_ref().unwrap(),
            assembly.decls.as_ref().unwrap(),
            assembly.defs.as_mut().unwrap())?;

        matcher::match_all(
            report,
            opts,
            assembly.ast.as_ref().unwrap(),
            assembly.defs.as_mut().unwrap())?;

        assembly.iterations_taken = Some(resolver::resolve_iteratively(
            report,
            opts,
            fileserver,
            assembly.ast.as_ref().unwrap(),
            assembly.decls.as_ref().unwrap(),
            assembly.defs.as_mut().unwrap(),
            10)?);

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


#[test]
fn test_new_asm() -> Result<(), ()>
{
    let mut report = diagn::Report::new();

    let mut fileserver = util::FileServerMock::new();

    fileserver.add("main.asm", r#"
        #include "include.asm"

        loop:
            jmp 0x6666
            jmp end
            lda 0x100 ; !!should error!!

        end = $
        endLen = $ - end
    "#);

    fileserver.add("include.asm", r#"
        #ruledef {
            hlt => 0xad @ $`8
            jmp {addr: u8} => 0xaa01 @ addr
            jmp {addr: u16} => {
                assert(addr < 0x100)
                0xaa02 @ addr
            }
            jmp {addr: u16} => {
                assert(addr >= 0x100)
                0xaa03 @ addr
            }

            ld {x: u8} => {
                assert(x >= 0x80)
                0xcc01 @ x
            }
            ld {x: u8} => {
                assert(x >= 0xc0)
                0xcc02 @ x
            }

            lda {x: u8} => 0xaa @ x
        }
    "#);

    let root_file = "main.asm";
    
    let mut fileserver = util::FileServerReal::new();
    let root_file = "examples/nes/main.asm";

    let opts = AssemblyOptions {
        debug_iterations: true,
    };

    let result = assemble(
        &mut report,
        &opts,
        &mut fileserver,
        &[root_file]);

    if let Some(iters) = result.iterations_taken
    {
        println!("resolved in {} iterations", iters);
    }

    if let Some(output) = result.output
    {
        println!(
            "{}",
            output.format_annotated(
                &mut fileserver,
                4,
                2));
    }
    
    report.print_all(&mut std::io::stderr(), &fileserver);
    Ok(())
}