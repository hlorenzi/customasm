use crate::*;


#[derive(Debug)]
pub struct Instruction
{
    pub item_ref: util::ItemRef<Self>,
    pub matches: asm::InstructionMatches,
    pub encoding_statically_known: bool,
    pub encoding: util::BigInt,
    pub resolved: bool,
}


pub fn define(
    _report: &mut diagn::Report,
    ast: &mut asm::AstTopLevel,
    _decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm::AstAny::Instruction(ast_instr) = any_node
        {
            let item_ref = defs.instructions.next_item_ref();

            let instr = Instruction {
                item_ref,
                matches: asm::InstructionMatches::new(),
                encoding_statically_known: false,
                encoding: util::BigInt::new(0, Some(0)),
                resolved: false,
            };
            
            defs.instructions.define(item_ref, instr);
                
            ast_instr.item_ref = Some(item_ref);
        }
    }


    Ok(())
}