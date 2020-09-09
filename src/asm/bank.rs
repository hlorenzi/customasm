use crate::*;


pub struct Bank
{
    pub name: String,
    
	pub addr: util::BigInt,
	pub size: Option<usize>,
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
            addr: util::BigInt::from(0),
            size: None,
            output_offset: Some(0),
            fill: false,
            decl_span: None,
        }
    }
}