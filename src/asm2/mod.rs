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
    Instruction,
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
};


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

    let mut run = || -> Result<(), ()>
    {
        let mut ast = parser::parse_and_resolve_includes(
            &mut report,
            &fileserver,
            root_file,
            &mut Vec::new())?;

        let mut decls = decls::collect(
            &mut report,
            &mut ast)?;
            
        let mut defs = defs::define(
            &mut report,
            &mut ast,
            &mut decls)?;
            
        resolver::resolve_constants(
            &mut report,
            &ast,
            &decls,
            &mut defs)?;

        matcher::match_all(
            &mut report,
            &ast,
            &mut defs)?;
    
        let iters = resolver::resolve_iteratively(
            &mut report,
            &ast,
            &decls,
            &mut defs,
            10)?;
            
        //println!("{:#?}", ast);
        //println!("{:#?}", decls);
        //println!("{:#?}", defs.instructions);
        //println!("{:#?}", defs.symbols);
        println!("resolved in {} iterations", iters);
        
        Ok(())
    };

    drop(run());
    report.print_all(&mut std::io::stderr(), &fileserver);
    Ok(())
}