use crate::*;


pub fn parse_directive_bits(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    state.asm_state.cur_wordsize = asm::parser::parse_expr_usize_fn(state, |u| match u
    {
        0 => None,
        _ => Some(u)
    })?;

    Ok(())
}


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


pub fn parse_directive_align(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let wordsize = asm::parser::parse_expr_usize_fn(state, |u| match u
    {
        0 => None,
        _ => Some(u),
    })?;

    let bankdata = state.asm_state.get_bankdata(state.asm_state.cur_bank);
    let skip_bits = bankdata.bits_until_aligned(state.asm_state, wordsize);

    let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
    bankdata.reserve(skip_bits);
    
    Ok(())
}


pub fn parse_directive_labelalign(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    state.asm_state.cur_labelalign = asm::parser::parse_expr_usize_fn(state, |u| match u
    {
        0 => None,
        _ => Some(u),
    })?;

    Ok(())
}


pub fn parse_directive_addr(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let (addr, addr_span) = asm::parser::parse_expr_bigint(state)?;

    let bankdata = state.asm_state.get_bankdata(state.asm_state.cur_bank);
    let skip_bits = bankdata.bits_until_address(
        state.asm_state,
        addr,
        state.report.clone(),
        &addr_span)?;

    let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
    bankdata.reserve_or_backtrack(skip_bits);
    
    Ok(())
}