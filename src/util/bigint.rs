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


    pub fn checked_to_usize(&self) -> Option<usize>
    {
        (&self.bigint).try_into().ok()
    }


    pub fn checked_to_isize(&self) -> Option<isize>
    {
        (&self.bigint).try_into().ok()
    }


    pub fn checked_div(&self, rhs: &BigInt) -> Option<BigInt>
    {
        self.bigint.checked_div(&rhs.bigint).map(|res| res.into())
    }


    pub fn checked_rem(&self, rhs: &BigInt) -> Option<BigInt>
    {
        if rhs.bigint == num_bigint::BigInt::from(0)
            { None }
        else
            { Some((&self.bigint % &rhs.bigint).into()) }
    }


    pub fn shl(&self, rhs: usize) -> BigInt
    {
        (&self.bigint << rhs).into()
    }


    pub fn shr(&self, rhs: usize) -> BigInt
    {
        let lhs_sign = self.bigint.sign();
        let result = &self.bigint >> rhs;

        if lhs_sign == num_bigint::Sign::Minus &&
            result.sign() == num_bigint::Sign::NoSign
        {
            BigInt::from(-1)
        }
        else
        {
            result.into()
        }
    }


    pub fn checked_shl(&self, rhs: &BigInt) -> Option<BigInt>
    {
        (&rhs.bigint)
            .try_into()
            .ok()
            .map(|rhs| self.shl(rhs).into())
    }
    
    
    pub fn checked_shr(&self, rhs: &BigInt) -> Option<BigInt>
    {
        (&rhs.bigint)
            .try_into()
            .ok()
            .map(|rhs| self.shr(rhs).into())
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
        let mut be_bytes = self.bigint.to_bytes_le().1;
        while be_bytes.len() < self.size.unwrap() / 8
        {
            be_bytes.push(0);
        }

        let new_value = num_bigint::BigInt::from_bytes_be(num_bigint::Sign::Plus, &be_bytes);
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


impl std::ops::Add for &BigInt
{
    type Output = BigInt;


    fn add(self, rhs: &BigInt) -> Self::Output
    {
        (&self.bigint + &rhs.bigint).into()
    }
}


impl std::ops::Sub for &BigInt
{
    type Output = BigInt;


    fn sub(self, rhs: &BigInt) -> Self::Output
    {
        (&self.bigint - &rhs.bigint).into()
    }
}


impl std::ops::Mul for &BigInt
{
    type Output = BigInt;


    fn mul(self, rhs: &BigInt) -> Self::Output
    {
        (&self.bigint * &rhs.bigint).into()
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