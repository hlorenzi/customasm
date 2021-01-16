use crate::*;


pub struct Bank
{
    pub name: String,
    
    pub wordsize: usize,
    pub labelalign: usize,
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
    pub invocations: Vec<asm::Invocation>,
}


impl Bank
{
    pub fn new_default() -> Bank
    {
        Bank {
            name: "".to_string(),
            wordsize: 8,
            labelalign: 0,
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
    pub fn check_writable(
        &self,
        state: &asm::State,
        report: diagn::RcReport,
        span: &diagn::Span)
        -> Result<(), ()>
    {
        let bank = &state.banks[self.bank_ref.index];
        if bank.output_offset.is_none()
        {
            report.error_span("current bank is non-writable (`#outp` is not set)", &span);
            return Err(());
        }

        Ok(())
    }


    pub fn push_invocation(&mut self, invoc: asm::Invocation)
    {
        self.cur_bit_offset += invoc.size_guess;
        self.invocations.push(invoc);
    }


    pub fn reserve(&mut self, bits: usize)
    {
        self.cur_bit_offset += bits;
    }


    pub fn reserve_or_backtrack(&mut self, bits: isize)
    {
        if bits < 0
        {
            self.cur_bit_offset -= (-bits) as usize;
        }
        else
        {
            self.cur_bit_offset += bits as usize;
        }
    }


    pub fn bits_until_aligned(&self, state: &asm::State, wordsize: usize) -> usize
    {
		let _bank = &state.banks[self.bank_ref.index];
		
        let excess_bits = self.cur_bit_offset % wordsize;
        if excess_bits != 0
        {
            wordsize - excess_bits
        }
        else
        {
            0
        }
    }


    pub fn bits_until_address(
        &self,
        state: &asm::State,
        addr: util::BigInt,
        report: diagn::RcReport,
        span: &diagn::Span)
        -> Result<isize, ()>
    {
        if addr.sign() < 0
        {
            report.error_span("address is out of valid range", span);
            return Err(());
        }

        let wordsize = state.cur_wordsize;
        
        let excess_bits = wordsize - (self.cur_bit_offset % wordsize);
        let excess_bits = if excess_bits != 0
        {
            wordsize - excess_bits
        }
        else
        {
            0
        };

		let bank = &state.banks[self.bank_ref.index];
			
		let addr_approx =
			&util::BigInt::from(self.cur_bit_offset / wordsize) +
            &bank.addr_start;

        if bank.output_offset.is_some() && &addr_approx > &addr
        {
            report.error_span("address is behind current position", span);
            return Err(());
        }

        match (&addr - &addr_approx).checked_to_isize()
        {
            None =>
            {
                report.error_span("address is out of valid range", span);
                return Err(());
            }

            Some(addr_diff) =>
            {
                // FIXME: operations can overflow
                Ok(addr_diff * (wordsize as isize) + (excess_bits as isize))
            }
        }
    }
}