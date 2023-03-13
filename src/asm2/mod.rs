use crate::*;


pub mod parser;
pub use parser::{
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
    AstNodeAny,
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
    RulePatternPart,
};

pub mod matcher;
pub use matcher::{
    InstructionMatches,
    InstructionMatch,
    InstructionArgument,
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
            jmp loop, 0x7777
    "#);

    fileserver.add("include.asm", r#"
        #ruledef {
            hlt => 0x1234
            hlt => 0x5678
            cld => 0x1111
            jmp {addr: u16} {addr2: u32} => 0xa0 @ addr
            jmp {addr: u16}, {addr2: u32} => 0xa1 @ addr
        }

        #ruledef hey {
            hlt => 0x9abc
            jmp {addr: u16} => 0xb0 @ addr
            jmp loop => 0xb1
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
            &decls,
            &defs)?;

        Ok(())
    };

    drop(run());
    report.print_all(&mut std::io::stderr(), &fileserver);
    Ok(())
}