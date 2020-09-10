use crate::*;


pub struct Bank
{
    pub name: String,
    
    pub wordsize: usize,
	pub addr_start: util::BigInt,
	pub addr_size: Option<usize>,
	pub output_offset: Option<usize>,
	pub fill: bool,
    pub decl_span: Option<diagn::Span>,
}


pub struct BankData
{
    pub bank_ref: asm::BankRef,
    pub cur_bit_offset: usize,
    pub rule_invokations: Vec<asm::RuleInvokation>,
}


impl Bank
{
    pub fn new_default() -> Bank
    {
        Bank {
            name: "".to_string(),
            wordsize: 8,
            addr_start: util::BigInt::from(0),
            addr_size: None,
            output_offset: Some(0),
            fill: false,
            decl_span: None,
        }
    }
}