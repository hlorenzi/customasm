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
    InstructionArgument,
    InstructionArgumentKind,
};

pub mod resolver;
pub use resolver::{
    ResolutionState,
};


#[test]
fn test_new_asm() -> Result<(), ()>
{
    let mut report = diagn::Report::new();

    let mut fileserver = util::FileServerMock::new();

    fileserver.add("main.asm", r#"
        #include "include.asm"

        res = w + ww + yy
        x = 0
        .a = 0x111
        .b = 0x222
        .c = 0x333
        .d = .a + .b + .c
        .e = x.a + x.b + x.c + y.e
        y = 1
        .a = 0x1000
        .b = 0x2000
        .c = 0x3000
        .d = .a + .b + .c
        .e = y.a + y.b + y.c
        z = 2
        w = x + y + z
        ww = x.a + x.b + x.c
        yy = y.a + y.b + y.c

        loop:
            hlt
            jmp loop
            jmp loop 0x6666
        .inner:
            jmp loop + 0x7777
            jmp a b
        ..inner:
            hlt
            ;xyz
            ;abc def + ghi
    "#);

    fileserver.add("include.asm", r#"
        #bankdef a {
            bits = 16 * 4
        }
        
        #ruledef {
            hlt => 0x1234
            hlt => 0x5678
            cld => 0x1111
            jmp {addr: u16} {addr2: u32} => 0xa0 @ addr
            jmp {addr: u16} + {addr2: u32} => 0xa1 @ addr
        }

        #ruledef inner {
            a => 0x99
            a => 0xdd
        }

        #ruledef hey {
            hlt => 0x9a
            jmp {addr: u16} => 0xb0 @ addr
            jmp loop => 0xb1
            jmp {arg: inner} b => 0x44 @ arg
        }
    "#);

    let mut run = || -> Result<(), ()>
    {
        let mut ast = parser::parse_and_resolve_includes(
            &mut report,
            &fileserver,
            "main.asm",
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
    
        println!("{:#?}", ast);
        println!("{:#?}", decls);
        println!("{:#?}", defs);
    
        resolver::resolve_once(
            &mut report,
            &ast,
            &decls,
            &mut defs)?;
            
        println!("{:#?}", ast);
        println!("{:#?}", decls);
        println!("{:#?}", defs);

        Ok(())
    };

    drop(run());
    report.print_all(&mut std::io::stderr(), &fileserver);
    Ok(())
}