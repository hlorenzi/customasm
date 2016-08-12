#[derive(Clone)]
pub struct Integer
{
	pub value: i64,
	pub explicit_width: Option<usize>
}


impl Integer
{
	pub fn new(value: i64) -> Integer
	{
		Integer
		{
			value: value,
			explicit_width: None
		}
	}
	
	
	pub fn new_with_explicit_width(value: i64, width: usize) -> Integer
	{
		Integer
		{
			value: value,
			explicit_width: Some(width)
		}
	}
	
	
	pub fn new_from_str(radix: usize, value_str: &str) -> Option<Integer>
	{
		match i64::from_str_radix(value_str, radix as u32)
		{
			Ok(value) => Some(Integer::new(value)),
			Err(_) => None
		}
	}
	
	
	pub fn get_bit(&self, index: usize) -> bool
	{
		(self.value & (1 << index)) != 0
	}
	
	
	pub fn get_width(&self) -> usize
	{
		match self.explicit_width
		{
			Some(width) => width,
			None => self.get_minimum_width()
		}
	}
	
	
	pub fn get_minimum_width(&self) -> usize
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


	pub fn checked_add(&self, other: &Integer) -> Option<Integer>
	{
		self.value.checked_add(other.value).map(|result| Integer
		{
			value: result,
			explicit_width: None
		})
	}
	
	
	pub fn checked_sub(&self, other: &Integer) -> Option<Integer>
	{
		self.value.checked_sub(other.value).map(|result| Integer
		{
			value: result,
			explicit_width: None
		})
	}
	
	
	pub fn checked_mul(&self, other: &Integer) -> Option<Integer>
	{
		self.value.checked_mul(other.value).map(|result| Integer
		{
			value: result,
			explicit_width: None
		})
	}
	
	
	pub fn checked_div(&self, other: &Integer) -> Option<Integer>
	{
		self.value.checked_div(other.value).map(|result| Integer
		{
			value: result,
			explicit_width: None
		})
	}
	
	
	pub fn checked_shl(&self, other: &Integer) -> Option<Integer>
	{
		self.value.checked_shl(other.value as u32).map(|result| Integer
		{
			value: result,
			explicit_width: None
		})
	}
	
	
	pub fn checked_shr(&self, other: &Integer) -> Option<Integer>
	{
		self.value.checked_shr(other.value as u32).map(|result| Integer
		{
			value: result,
			explicit_width: None
		})
	}
	
	
	pub fn bit_and(&self, other: &Integer) -> Option<Integer>
	{
		Some(Integer
		{
			value: self.value & other.value,
			explicit_width: None
		})
	}
	
	
	pub fn bit_or(&self, other: &Integer) -> Option<Integer>
	{
		Some(Integer
		{
			value: self.value | other.value,
			explicit_width: None
		})
	}
	
	
	pub fn bit_xor(&self, other: &Integer) -> Option<Integer>
	{
		Some(Integer
		{
			value: self.value ^ other.value,
			explicit_width: None
		})
	}
	
	
	pub fn slice(&self, left: usize, right: usize) -> Integer
	{
		let mut value: i64 = 0;		
		let width = left - right + 1;
		
		for i in right..(left + 1)
			{ value |= (self.value & (1 << i)) >> right; }
		
		Integer
		{
			value: value,
			explicit_width: Some(width)
		}
	}
	
	
	pub fn eq(&self, other: &Integer) -> bool
	{
		self.value == other.value
	}
	
	
	pub fn ne(&self, other: &Integer) -> bool
	{
		self.value != other.value
	}
	
	
	pub fn lt(&self, other: &Integer) -> bool
	{
		self.value < other.value
	}
	
	
	pub fn gt(&self, other: &Integer) -> bool
	{
		self.value > other.value
	}
	
	
	pub fn le(&self, other: &Integer) -> bool
	{
		self.value <= other.value
	}
	
	
	pub fn ge(&self, other: &Integer) -> bool
	{
		self.value >= other.value
	}
}