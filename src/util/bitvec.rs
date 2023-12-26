use crate::*;

#[derive(Debug)]
pub struct BitVec
{
    data: util::BigInt,
    len: usize,
    pub spans: Vec<BitVecSpan>,
}

#[derive(Clone, Debug)]
pub struct BitVecSpan
{
    pub addr: util::BigInt,
    pub offset: Option<usize>,
    pub size: usize,
    pub span: diagn::Span,
}

impl BitVec
{
    pub fn new() -> BitVec
    {
        BitVec {
            data: util::BigInt::from(0),
            len: 0,
            spans: Vec::new(),
        }
    }

    pub fn write_bit(&mut self, index: usize, value: bool)
    {
        self.data.set_bit(index, value);

        if index + 1 > self.len
        {
            self.len = index + 1;
        }
    }

    pub fn read_bit(&self, index: usize) -> bool
    {
        self.data.get_bit(index)
    }

    pub fn len(&self) -> usize
    {
        self.len
    }

    pub fn write_bigint(&mut self, index: usize, bigint: &util::BigInt)
    {
        let size = bigint.size.unwrap();

        for i in 0..size
        {
            self.data.set_bit(index + i, bigint.get_bit(size - 1 - i));
        }

        if index + size > self.len
        {
            self.len = index + size;
        }
    }

    pub fn write_bigint_with_span(
        &mut self,
        span: diagn::Span,
        offset: usize,
        addr: util::BigInt,
        bigint: &util::BigInt,
    )
    {
        self.write_bigint(offset, bigint);

        self.mark_span(Some(offset), bigint.size.unwrap(), addr, span);
    }

    pub fn mark_span(
        &mut self,
        offset: Option<usize>,
        size: usize,
        addr: util::BigInt,
        span: diagn::Span,
    )
    {
        self.spans.push(BitVecSpan {
            offset,
            size,
            addr,
            span,
        });
    }

    pub fn to_bigint(&self) -> util::BigInt
    {
        let mut bigint = util::BigInt::from(0);

        for i in 0..self.len
        {
            bigint.set_bit(self.len - 1 - i, self.read_bit(i));
        }

        bigint.size = Some(self.len);
        bigint
    }
}

impl std::fmt::LowerHex for BitVec
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        use std::fmt::Write;

        let mut i = 0;
        while i < self.len()
        {
            let mut digit = 0;
            for _ in 0..4
            {
                digit <<= 1;
                digit |= if self.read_bit(i) { 1 } else { 0 };
                i += 1;
            }

            let c = if digit < 10
            {
                ('0' as u8 + digit) as char
            }
            else
            {
                ('a' as u8 + digit - 10) as char
            };

            f.write_char(c)?;
        }

        Ok(())
    }
}
