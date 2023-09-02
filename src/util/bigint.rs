use crate::*;


pub const BIGINT_MAX_BITS: u64 = 8 * 100_000_000;


#[derive(Clone, Eq)]
pub struct BigInt
{
    bigint: num_bigint::BigInt,
    pub size: Option<usize>,
}


impl BigInt
{
    pub fn new<T>(
        value: T,
        size: Option<usize>)
        -> BigInt
        where T: Into<num_bigint::BigInt>
    {
        BigInt {
            bigint: value.into(),
            size,
        }
    }


    pub fn as_string(&self) -> String
    {
        String::from_utf8_lossy(&self.bigint.to_signed_bytes_be()).to_string()
    }


    pub fn from_bytes_be(bytes: &[u8]) -> BigInt
    {
        let bigint = num_bigint::BigInt::from_signed_bytes_be(&bytes);
        BigInt {
            bigint,
            size: Some(bytes.len() * 8),
        }
    }


    pub fn set_bit(&mut self, index: usize, value: bool)
    {
        self.bigint.set_bit(index.try_into().unwrap(), value)
    }


    pub fn get_bit(&self, index: usize) -> bool
    {
        self.bigint.bit(index.try_into().unwrap())
    }


    pub fn min_size(&self) -> usize
    {
        if self.bigint.sign() == num_bigint::Sign::NoSign
            { return 1; }
    
        if self.bigint < num_bigint::BigInt::from(0)
        {
            let y: num_bigint::BigInt = &self.bigint + 1;
            (y.bits() + 1).try_into().unwrap()
        }
        else
            { self.bigint.bits().try_into().unwrap() }
    }


    pub fn size_or_min_size(&self) -> usize
    {
        if let Some(size) = self.size
        {
            size
        }
        else
        {
            self.min_size()
        }
    }


    pub fn sign(&self) -> isize
    {
        match self.bigint.sign()
        {
            num_bigint::Sign::Minus => -1,
            num_bigint::Sign::NoSign => 0,
            num_bigint::Sign::Plus => 1,
        }
    }


    pub fn maybe_into<T>(&self) -> Option<T>
        where T: for<'a> TryFrom<&'a num_bigint::BigInt>
    {
        (&self.bigint).try_into().ok()
    }


    pub fn checked_into<T>(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span)
        -> Result<T, ()>
        where T: for<'a> TryFrom<&'a num_bigint::BigInt>
    {
        match self.maybe_into::<T>()
        {
            Some(res) => Ok(res),
            None =>
            {            
                report.error_span(
                    "value is out of supported range",
                    span);
                
                Err(())
            }
        }
    }


    pub fn checked_into_nonzero_usize(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span)
        -> Result<usize, ()>
    {
        match self.maybe_into::<usize>()
        {
            None | Some(0) =>
            {            
                report.error_span(
                    "value is out of supported range",
                    span);
                
                Err(())
            }
            Some(res) => Ok(res),
        }
    }


    pub fn checked_add(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span,
        rhs: &BigInt)
        -> Result<BigInt, ()>
    {
        let largest_bits = std::cmp::max(
            self.bigint.bits(),
            rhs.bigint.bits());
            
        if largest_bits >= BIGINT_MAX_BITS - 1
        {
            report.error_span(
                "value is out of supported range",
                span);
            
            return Err(());
        }

        self.bigint
            .checked_add(&rhs.bigint)
            .ok_or(())
            .map(|res| res.into())
    }


    pub fn checked_sub(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span,
        rhs: &BigInt)
        -> Result<BigInt, ()>
    {
        let largest_bits = std::cmp::max(
            self.bigint.bits(),
            rhs.bigint.bits());
            
        if largest_bits >= BIGINT_MAX_BITS - 2
        {
            report.error_span(
                "value is out of supported range",
                span);
            
            return Err(());
        }

        self.bigint
            .checked_sub(&rhs.bigint)
            .ok_or(())
            .map(|res| res.into())
    }


    pub fn checked_mul(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span,
        rhs: &BigInt)
        -> Result<BigInt, ()>
    {
        let largest_bits = std::cmp::max(
            self.bigint.bits(),
            rhs.bigint.bits());
            
        if largest_bits >= BIGINT_MAX_BITS / 2
        {
            report.error_span(
                "value is out of supported range",
                span);
            
            return Err(());
        }

        self.bigint
            .checked_mul(&rhs.bigint)
            .ok_or(())
            .map(|res| res.into())
    }


    pub fn checked_div(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span,
        rhs: &BigInt)
        -> Result<BigInt, ()>
    {
        if rhs.bigint == num_bigint::BigInt::from(0)
        {
            report.error_span(
                "division by zero",
                span);
            
            return Err(());
        }

        self.bigint
            .checked_div(&rhs.bigint)
            .ok_or(())
            .map(|res| res.into())
    }


    pub fn checked_mod(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span,
        rhs: &BigInt)
        -> Result<BigInt, ()>
    {
        if rhs.bigint == num_bigint::BigInt::from(0)
        {
            report.error_span(
                "modulo by zero",
                span);
            
            return Err(());
        }

        Ok((&self.bigint % &rhs.bigint).into())
    }


    pub fn checked_shl(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span,
        rhs: &BigInt)
        -> Result<BigInt, ()>
    {
        let maybe_rhs_u64: Result<u32, _> = (&rhs.bigint).try_into();

        let result_too_large = {
            match maybe_rhs_u64
            {
                Err(_) => true,
                Ok(size) =>
                    self.bigint.bits() + (size as u64) >= BIGINT_MAX_BITS,
            }
        };

        if result_too_large
        {
            report.error_span(
                "value is out of supported range",
                span);
            
            return Err(());
        }

        (&rhs.bigint)
            .try_into()
            .map(|rhs: usize| (&self.bigint << rhs).into())
            .map_err(|_| ())
    }
    
    
    pub fn checked_shr(
        &self,
        report: &mut diagn::Report,
        span: diagn::Span,
        rhs: &BigInt)
        -> Result<BigInt, ()>
    {
        let maybe_result: Result<num_bigint::BigInt, ()> = (&rhs.bigint)
            .try_into()
            .map(|rhs: usize| (&self.bigint >> rhs).into())
            .map_err(|_| ());

        match maybe_result
        {
            Ok(result) => Ok(result.into()),

            Err(_) =>
            {
                report.error_span(
                    "value is out of supported range",
                    span);
                
                Err(())
            }
        }
    }
    
    
    pub fn slice(
        &self,
        left: usize,
        right: usize)
        -> BigInt
    {
        let mut result = BigInt::from(0);

        for i in 0..(left - right)
        {
            result.set_bit(
                i,
                self.get_bit(right + i));
        }

        result.size = Some(left - right);
        result
    }
    
    
    pub fn concat(
        &self,
        lhs_slice: (usize, usize),
        rhs: &BigInt,
        rhs_slice: (usize, usize))
        -> BigInt
    {
        let lhs_size = lhs_slice.0 - lhs_slice.1;
        let rhs_size = rhs_slice.0 - rhs_slice.1;

        let mut result = BigInt::from(0);

        for i in 0..(lhs_slice.0 - lhs_slice.1)
        {
            result.set_bit(
                i + rhs_size,
                self.get_bit(lhs_slice.1 + i));
        }

        for i in 0..(rhs_slice.0 - rhs_slice.1)
        {
            result.set_bit(
                i,
                rhs.get_bit(rhs_slice.1 + i));
        }

        result.size = Some(lhs_size + rhs_size);
        result
    }


    pub fn convert_le(&self) -> BigInt
    {
        let Some(size) = self.size
            else { panic!("attempting `le` conversion on an unsized value") };

        // Slice is needed here for negative numbers
        let value = self.slice(size, 0);
        
        let mut be_bytes = value.bigint.to_bytes_le().1;
        while be_bytes.len() < size / 8
        {
            be_bytes.push(0);
        }

        let new_value = num_bigint::BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &be_bytes);
        
        BigInt::new(new_value, self.size)
    }
}


impl std::fmt::Debug for BigInt
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str(&format!("{:#x}", self.bigint))?;

        if let Some(size) = self.size
        {
            f.write_str(&format!("`{}", size))?;
        }

        Ok(())
    }
}


impl<T: Into<num_bigint::BigInt>> From<T> for BigInt
{
    fn from(value: T) -> BigInt
    {
        BigInt
        {
            bigint: value.into(),
            size: None,
        }
    }
}


impl std::ops::Not for &BigInt
{
    type Output = BigInt;


    fn not(self) -> Self::Output
    {
        let mut x_bytes = self.bigint.to_signed_bytes_le();
        
        if self.bigint.sign() != num_bigint::Sign::Minus
            { x_bytes.push(0); }
        
        for i in 0..x_bytes.len()
            { x_bytes[i] = !x_bytes[i]; }
        
        num_bigint::BigInt::from_signed_bytes_le(&x_bytes).into()
    }
}


impl std::ops::Neg for &BigInt
{
    type Output = BigInt;


    fn neg(self) -> Self::Output
    {
        (-&self.bigint).into()
    }
}


impl std::ops::BitAnd for &BigInt
{
    type Output = BigInt;


    fn bitand(self, rhs: &BigInt) -> Self::Output
    {
        (&self.bigint & &rhs.bigint).into()
    }
}


impl std::ops::BitOr for &BigInt
{
    type Output = BigInt;


    fn bitor(self, rhs: &BigInt) -> Self::Output
    {
        (&self.bigint | &rhs.bigint).into()
    }
}


impl std::ops::BitXor for &BigInt
{
    type Output = BigInt;


    fn bitxor(self, rhs: &BigInt) -> Self::Output
    {
        (&self.bigint ^ &rhs.bigint).into()
    }
}


impl std::cmp::PartialEq for BigInt
{
    fn eq(&self, rhs: &BigInt) -> bool
    {
        self.bigint.partial_cmp(&rhs.bigint) == Some(std::cmp::Ordering::Equal)
    }
}


impl std::cmp::PartialOrd for BigInt
{
    fn partial_cmp(&self, rhs: &BigInt) -> Option<std::cmp::Ordering>
    {
        self.bigint.partial_cmp(&rhs.bigint)
    }
}


impl std::cmp::Ord for BigInt
{
    fn cmp(&self, rhs: &BigInt) -> std::cmp::Ordering
    {
        self.bigint.cmp(&rhs.bigint)
    }
}


impl std::fmt::LowerHex for BigInt
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        self.bigint.fmt(f)
    }
}