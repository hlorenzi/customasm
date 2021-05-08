use num_bigint;


#[derive(Clone, Debug, Eq)]
pub struct BigInt
{
    bigint: num_bigint::BigInt,
    pub size: Option<usize>,
}


impl BigInt
{
    pub fn new<T>(bigint: T, size: Option<usize>) -> BigInt
    where T: Into<num_bigint::BigInt>
    {
        BigInt
        {
            bigint: bigint.into(),
            size,
        }
    }


    pub fn new_from_str(s: &str) -> BigInt
    {
        let bytes = s.bytes().collect::<Vec<u8>>();
        let bigint = num_bigint::BigInt::from_signed_bytes_be(&bytes);
        BigInt
        {
            bigint,
            size: Some(bytes.len() * 8),
        }
    }


    pub fn as_string(&self) -> String
    {
        String::from_utf8_lossy(&self.bigint.to_signed_bytes_be()).to_string()
    }


    pub fn from_bytes_be(bytes: &[u8]) -> BigInt
    {
        let bigint = num_bigint::BigInt::from_signed_bytes_be(&bytes);
        BigInt
        {
            bigint,
            size: Some(bytes.len() * 8),
        }
    }


    pub fn set_bit(&self, index: usize, value: bool) -> BigInt
    {
        if value
        {
            self | &Into::<BigInt>::into(1).shl(index)
        }
        else
        {
            self & &!&Into::<BigInt>::into(0).shl(index)
        }
    }


    pub fn get_bit(&self, index: usize) -> bool
    {
        let bytes = self.bigint.to_signed_bytes_le();
        
        let byte_index = index / 8;
        if byte_index >= bytes.len()
            { return self.bigint.sign() == num_bigint::Sign::Minus; }
            
        let mut byte = bytes[byte_index];
        
        let mut bit_index = index % 8;
        while bit_index > 0
        {
            byte >>= 1;
            bit_index -= 1;
        }
        
        (byte & 0b1) != 0
    }


    pub fn min_size(&self) -> usize
    {
        use num_traits::Zero;

        if self.bigint.is_zero()
            { return 1; }
    
        if self.bigint < num_bigint::BigInt::zero()
        {
            let y: num_bigint::BigInt = &self.bigint + 1;
            y.bits() + 1
        }
        else
            { self.bigint.bits() }
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
        use num_traits::ToPrimitive;
        self.bigint.to_usize()
    }


    pub fn checked_to_isize(&self) -> Option<isize>
    {
        use num_traits::ToPrimitive;
        self.bigint.to_isize()
    }


    pub fn checked_div(&self, rhs: &BigInt) -> Option<BigInt>
    {
        self.bigint.checked_div(&rhs.bigint).map(|res| res.into())
    }


    pub fn checked_rem(&self, rhs: &BigInt) -> Option<BigInt>
    {
        use num_traits::Zero;
        if rhs.bigint == num_bigint::BigInt::zero()
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

        if lhs_sign == num_bigint::Sign::Minus && result.sign() == num_bigint::Sign::NoSign
            { BigInt::from(-1) }
        else
            { result.into() }
    }


    pub fn checked_shl(&self, rhs: &BigInt) -> Option<BigInt>
    {
        use num_traits::ToPrimitive;

        rhs.bigint.to_usize().map(|rhs| self.shl(rhs).into())
    }
    
    
    pub fn checked_shr(&self, rhs: &BigInt) -> Option<BigInt>
    {
        use num_traits::ToPrimitive;

        rhs.bigint.to_usize().map(|rhs| self.shr(rhs).into())
    }
    
    
    pub fn concat(&self, lhs_slice: (usize, usize), rhs: &BigInt, rhs_slice: (usize, usize)) -> BigInt
    {
        let lhs_size = lhs_slice.0 - lhs_slice.1;
        let rhs_size = rhs_slice.0 - rhs_slice.1;
        let lhs = self.slice(lhs_slice.0, lhs_slice.1).shl(rhs_size);
        let rhs = rhs.slice(rhs_slice.0, rhs_slice.1);

        let mut result: BigInt = (&lhs | &rhs).into();
        result.size = Some(lhs_size + rhs_size);
        result
    }
    
    
    pub fn slice(&self, left: usize, right: usize) -> BigInt
    {
        use num_traits::Zero;
        use num_traits::One;

        let mut mask = num_bigint::BigInt::zero();
        for _ in 0..(left - right)
            { mask = (mask << 1) + num_bigint::BigInt::one(); }
        
        let shifted_mask = BigInt::new(mask, None).shl(right);
        let mut result = (self & &shifted_mask).shr(right);
        result.size = Some(left - right);
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


    fn combine_bits<F>(&self, rhs: &BigInt, f: F) -> BigInt
    where F: Fn(u8, u8) -> u8
    {
        let mut lhs_bytes = self.bigint.to_signed_bytes_le();
        let mut lhs_sign = self.bigint.sign();
        let mut rhs_bytes = rhs.bigint.to_signed_bytes_le();
        let mut rhs_sign = rhs.bigint.sign();
        
        if lhs_sign != num_bigint::Sign::Minus
            { lhs_bytes.push(0); }
        
        if rhs_sign != num_bigint::Sign::Minus
            { rhs_bytes.push(0); }
            
        if rhs_bytes.len() > lhs_bytes.len()
        {
            std::mem::swap(&mut lhs_bytes, &mut rhs_bytes);
            std::mem::swap(&mut lhs_sign, &mut rhs_sign);
        }
        
        for i in 0..lhs_bytes.len()
        {
            let rhs_byte = if i < rhs_bytes.len()
                { rhs_bytes[i] }
            else if rhs_sign == num_bigint::Sign::Minus
                { 0xff }
            else
                { 0 };
            
            lhs_bytes[i] = f(lhs_bytes[i], rhs_byte);
        }
        
        num_bigint::BigInt::from_signed_bytes_le(&lhs_bytes).into()
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
        self.combine_bits(rhs, |a, b| a & b).into()
    }
}


impl std::ops::BitOr for &BigInt
{
    type Output = BigInt;


    fn bitor(self, rhs: &BigInt) -> Self::Output
    {
        self.combine_bits(rhs, |a, b| a | b).into()
    }
}


impl std::ops::BitXor for &BigInt
{
    type Output = BigInt;


    fn bitxor(self, rhs: &BigInt) -> Self::Output
    {
        self.combine_bits(rhs, |a, b| a ^ b).into()
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