use crate::*;


pub struct Bank
{
    pub cur_bit_offset: usize,
    pub rule_invokations: Vec<asm::RuleInvokation>,
}