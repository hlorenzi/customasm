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
    pub invokations: Vec<asm::Invokation>,
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


impl BankData
{
    pub fn push_invokation(&mut self, invok: asm::Invokation)
    {
        self.cur_bit_offset += invok.size_guess;
        self.invokations.push(invok);
    }
}