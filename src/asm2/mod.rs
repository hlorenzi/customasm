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
};


#[test]
fn test_ast() -> Result<(), ()>
{
    let mut report = diagn::Report::new();

    let mut fileserver = util::FileServerMock::new();

    fileserver.add("main.asm", r#"
        #include "include.asm"

        loop:
            jmp loop
    "#);

    fileserver.add("include.asm", r#"
        #ruledef {
            jmp {addr: u16} {addr2: u32} => 0x55 @ addr
        }

        #ruledef hey {
            jmp {addr: u16} => 0x55 @ addr
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

        let decls = decls::collect(
            &mut report,
            &mut ast)?;
            
        println!("{:#?}", decls);

        let defs = defs::resolve(
            &mut report,
            &ast,
            &decls)?;
            
        println!("{:#?}", defs);

        Ok(())
    };

    drop(run());
    report.print_all(&mut std::io::stderr(), &fileserver);
    Ok(())
}