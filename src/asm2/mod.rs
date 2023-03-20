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

        ;res = w + ww + yy
        ;x = 0
        ;.a = 0x111
        ;.b = 0x222
        ;.c = 0x333
        ;.d = .a + .b + .c
        ;.e = x.a + x.b + x.c + y.e
        ;y = 1
        ;.a = 0x1000
        ;.b = 0x2000
        ;.c = 0x3000
        ;.d = .a + .b + .c
        ;.e = y.a + y.b + y.c
        ;z = 2
        ;w = x + y + z
        ;ww = x.a + x.b + x.c
        ;yy = y.a + y.b + y.c
        ;zz = loop + loop.inner + loop.inner.inner

        loop:
            ;hlt
            ;jmp 0x55
            ;jmp 0x6666
            jmp end

        end = $
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
            
        let mut defs = defs::resolve(
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
    
        resolver::resolve_iteratively(
            &mut report,
            &ast,
            &decls,
            &mut defs,
            3)?;
            
        //println!("{:#?}", ast);
        //println!("{:#?}", decls);
        println!("{:#?}", defs);

        Ok(())
    };

    drop(run());
    report.print_all(&mut std::io::stderr(), &fileserver);
    Ok(())
}