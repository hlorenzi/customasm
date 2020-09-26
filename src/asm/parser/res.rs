use crate::*;


pub fn parse_directive_res(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let words = asm::parser::parse_expr_usize(state)?;

    // FIXME: multiplication can overflow
    let bits = words * state.asm_state.cur_wordsize;

    let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
    bankdata.reserve(bits);
    
    Ok(())
}