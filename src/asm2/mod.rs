use crate::*;


pub mod parser;
pub use parser::{
    AstAny,
    AstConstant,
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
    AstLabel,
    AstRule,
    AstRuleParameter,
    AstRuleParameterType,
    AstRulePatternPart,
    AstTopLevel,
};

pub mod decls;
pub use decls::{
    ItemDecls,
    ItemRef,
};

pub mod defs;
pub use defs::{
    ItemDefs,
    Ruledef,
    Rule,
    RuleParameter,
    RuleParameterType,
    RulePattern,
    RulePatternPart,
};

pub mod matcher;
pub use matcher::{
    InstructionMatches,
    InstructionMatch,
    InstructionArgument,
    InstructionArgumentKind,
};


#[test]
fn test_new_asm() -> Result<(), ()>
{
    let mut report = diagn::Report::new();

    let mut fileserver = util::FileServerMock::new();

    fileserver.add("main.asm", r#"
        #include "include.asm"

        loop:
            hlt
            jmp loop
            jmp loop 0x6666
            jmp loop + 0x7777
            jmp a b
            xyz
            abc def + ghi
    "#);

    fileserver.add("include.asm", r#"
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
            hlt => 0x9abc
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

        println!("{:#?}", ast);

        let mut decls = decls::collect(
            &mut report,
            &mut ast)?;
            
        println!("{:#?}", decls);

        let defs = defs::resolve(
            &mut report,
            &ast,
            &mut decls)?;
            
        println!("{:#?}", defs);

        matcher::match_all(
            &mut report,
            &mut ast,
            &defs)?;

        Ok(())
    };

    drop(run());
    report.print_all(&mut std::io::stderr(), &fileserver);
    Ok(())
}