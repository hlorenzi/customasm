use crate::*;


#[derive(Debug)]
pub struct Instruction
{
    pub item_ref: util::ItemRef<Self>,
    pub matches: asm2::InstructionMatches,
    pub position_within_bank: usize,
    pub encoding: util::BigInt,
}


pub fn define(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::Instruction(ref mut ast_instr) = any_node
        {
            let item_ref = defs.instructions.next_item_ref();

            let instr = Instruction {
                item_ref,
                matches: asm2::InstructionMatches::new(),
                position_within_bank: 0,
                encoding: util::BigInt::new(0, Some(0)),
            };
            
            defs.instructions.define(item_ref, instr);
                
            ast_instr.item_ref = Some(item_ref);
        }
    }


    Ok(())
}