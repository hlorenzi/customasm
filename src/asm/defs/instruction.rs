use crate::*;


#[derive(Debug)]
pub struct Instruction
{
    pub item_ref: util::ItemRef<Self>,
    pub matches: asm::InstructionMatches,
    pub encoding: util::BigInt,
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
        if let asm::AstAny::Instruction(ref mut ast_instr) = any_node
        {
            let item_ref = defs.instructions.next_item_ref();

            let instr = Instruction {
                item_ref,
                matches: asm::InstructionMatches::new(),
                encoding: util::BigInt::new(0, Some(0)),
            };
            
            defs.instructions.define(item_ref, instr);
                
            ast_instr.item_ref = Some(item_ref);
        }
    }


    Ok(())
}