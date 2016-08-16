/// Currently only supports machine-sized operations.
#[derive(Clone)]
pub struct BigInt
{
	value: i64
}


impl BigInt
{
	pub fn from_i64(value: i64) -> BigInt
	{
		BigInt
		{
			value: value
		}
	}
	
	
	pub fn from_usize(value: usize) -> BigInt
	{
		BigInt
		{
			value: value as i64
		}
	}
	
	
	pub fn to_i64(&self) -> Option<i64>
	{
		Some(self.value as i64)
	}
	
	
	pub fn to_usize(&self) -> Option<usize>
	{
		if self.value < 0 || self.value > 0xff_ffff_ffff_ffff
			{ None }
		else
			{ Some(self.value as usize) }
	}
	
	
	pub fn from_str_radix(radix: usize, value_str: &str) -> Option<BigInt>
	{
		match i64::from_str_radix(value_str, radix as u32)
		{
			Ok(value) => Some(BigInt::from_i64(value)),
			Err(_) => None
		}
	}
	
	
	pub fn bit_at(&self, index: usize) -> bool
	{
		(self.value & (1 << index)) != 0
	}
	
	
	pub fn width(&self) -> usize
	{
		let mut value = self.value;
		let mut width = 0;
		
		if value >= 0
		{
			while value > 0
			{
				value >>= 1;
				width += 1;
			}
		}
		else
		{
			while value < -1
			{
				value >>= 1;
				width += 1;
			}
		}
		
		if width == 0
			{ width = 1; }
		
		width
	}


	pub fn checked_add(&self, other: &BigInt) -> Option<BigInt>
	{
		self.value.checked_add(other.value).map(|result| BigInt
		{
			value: result
		})
	}
	
	
	pub fn checked_sub(&self, other: &BigInt) -> Option<BigInt>
	{
		self.value.checked_sub(other.value).map(|result| BigInt
		{
			value: result
		})
	}
	
	
	pub fn checked_mul(&self, other: &BigInt) -> Option<BigInt>
	{
		self.value.checked_mul(other.value).map(|result| BigInt
		{
			value: result
		})
	}
	
	
	pub fn checked_div(&self, other: &BigInt) -> Option<BigInt>
	{
		self.value.checked_div(other.value).map(|result| BigInt
		{
			value: result
		})
	}
	
	
	pub fn checked_shl(&self, other: &BigInt) -> Option<BigInt>
	{
		self.value.checked_shl(other.value as u32).map(|result| BigInt
		{
			value: result
		})
	}
	
	
	pub fn checked_shr(&self, other: &BigInt) -> Option<BigInt>
	{
		self.value.checked_shr(other.value as u32).map(|result| BigInt
		{
			value: result
		})
	}
	
	
	pub fn bit_and(&self, other: &BigInt) -> Option<BigInt>
	{
		Some(BigInt
		{
			value: self.value & other.value
		})
	}
	
	
	pub fn bit_or(&self, other: &BigInt) -> Option<BigInt>
	{
		Some(BigInt
		{
			value: self.value | other.value
		})
	}
	
	
	pub fn bit_xor(&self, other: &BigInt) -> Option<BigInt>
	{
		Some(BigInt
		{
			value: self.value ^ other.value
		})
	}
	
	
	pub fn slice(&self, left: usize, right: usize) -> Option<BigInt>
	{
		let mut value: i64 = 0;
		
		for i in right..(left + 1)
			{ value |= (self.value & (1 << i)) >> right; }
		
		Some(BigInt
		{
			value: value
		})
	}
	
	
	pub fn eq(&self, other: &BigInt) -> bool
	{
		self.value == other.value
	}
	
	
	pub fn ne(&self, other: &BigInt) -> bool
	{
		self.value != other.value
	}
	
	
	pub fn lt(&self, other: &BigInt) -> bool
	{
		self.value < other.value
	}
	
	
	pub fn gt(&self, other: &BigInt) -> bool
	{
		self.value > other.value
	}
	
	
	pub fn le(&self, other: &BigInt) -> bool
	{
		self.value <= other.value
	}
	
	
	pub fn ge(&self, other: &BigInt) -> bool
	{
		self.value >= other.value
	}
}