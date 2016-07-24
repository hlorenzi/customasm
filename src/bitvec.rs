use std::fmt;


#[derive(Clone)]
pub struct BitVec
{
	bits: Vec<bool>
}


impl BitVec
{
	pub fn new() -> BitVec
	{
		BitVec
		{
			bits: Vec::new()
		}
	}
	
	
	pub fn new_from_vec(bits: Vec<bool>) -> BitVec
	{
		BitVec
		{
			bits: bits
		}
	}
	
	
	pub fn new_from_usize(mut value: usize) -> BitVec
	{
		let mut bitvec = BitVec::new();
		while value != 0
		{
			bitvec.insert_bit(0, value & 1 != 0);
			value >>= 1;
		}
		
		bitvec
	}


	pub fn new_from_str(radix: usize, value_str: &str) -> Result<BitVec, String>
	{	
		let mut bitvec = BitVec::new();
		match radix
		{
			10 =>
			{
				let mut value = match value_str.parse::<u64>()
				{
					Ok(x) => x,
					Err(_) => return Err(format!("decimal value `{}` is too large", value_str))
				};
				
				while value != 0
				{
					bitvec.insert_bit(0, value & 1 != 0);
					value >>= 1;
				}
			}
			
			16 =>
			{
				for c in value_str.chars()
				{
					let mut nibble = c.to_digit(16).unwrap();
					for _ in 0..4
					{
						bitvec.push_bit(nibble & 0b1000 != 0);
						nibble <<= 1;
					}
				}
			}
			
			2 =>
			{
				for c in value_str.chars()
					{ bitvec.push_bit(c == '1'); }
			}
			
			_ => unreachable!()
		}
		
		bitvec.trim();		
		Ok(bitvec)
	}


	pub fn new_from_str_sized(bit_num: usize, radix: usize, value_str: &str) -> Result<BitVec, String>
	{
		let mut bitvec = try!(BitVec::new_from_str(radix, value_str));
		
		if bitvec.len() > bit_num
			{ return Err(format!("value `{}` does not fit given size of `{}`", value_str, bit_num)); }
			
		bitvec.zero_extend(bit_num);
		Ok(bitvec)
	}
	
	
	pub fn len(&self) -> usize
	{
		self.bits.len()
	}
	
	
	pub fn get_vec(&self) -> &Vec<bool>
	{
		&self.bits
	}
	
	
	pub fn get_bit(&self, index: usize) -> bool
	{
		if index >= self.bits.len()
			{ false }
		else
			{ self.bits[index] }
	}
	
	
	pub fn extract(&self, left: usize, right: usize) -> BitVec
	{
		let mut result = BitVec::new();
		
		if left < right
		{
			for i in left..right
				{ result.push_bit(self.get_bit(i)) }
		}
		else
		{
			for i in right..left
				{ result.push_bit(self.get_bit(i)) }
		}
		
		result
	}
	
	
	pub fn set_bit(&mut self, index: usize, value: bool)
	{
		while self.bits.len() <= index
			{ self.bits.push(false); }
		
		self.bits[index] = value;
	}
	
	
	pub fn push_bit(&mut self, value: bool)
	{
		self.bits.push(value);
	}
	
	
	pub fn insert_bit(&mut self, index: usize, value: bool)
	{
		self.bits.insert(index, value);
	}
	
	
	pub fn remove_bit(&mut self, index: usize)
	{
		self.bits.remove(index);
	}
	
	
	pub fn trim(&mut self)
	{
		while self.len() > 1 && !self.get_bit(0)
			{ self.remove_bit(0); }
	}
	
	
	pub fn zero_extend(&mut self, final_bit_num: usize)
	{
		while self.bits.len() < final_bit_num
			{ self.bits.insert(0, false); }
	}
	
	
	pub fn set(&mut self, index: usize, other: &BitVec)
	{
		for i in 0..other.len()
			{ self.set_bit(index + i, other.get_bit(i)); }
	}
	
	
	pub fn push(&mut self, other: &BitVec)
	{
		for i in 0..other.len()
			{ self.push_bit(other.get_bit(i)); }
	}
	
	
	pub fn get_byte(&self, byte_index: usize) -> u8
	{
		let mut value = 0;
		for bit_index in 0..8
		{
			if byte_index * 8 + bit_index >= self.bits.len()
				{ break; }
			
			let bit_value = 
				if self.bits[byte_index * 8 + bit_index] { 1 } else { 0 };
			
			value |= bit_value << (7 - bit_index);
		}
		
		value
	}
	
	
	pub fn get_bytes(&self) -> Vec<u8>
	{
		let mut result = Vec::new();
		let mut byte_index = 0;
		
		while byte_index <= self.bits.len() / 8
		{
			result.push(self.get_byte(byte_index));
			byte_index += 1;
		}
		
		result
	}
	
		
	pub fn get_hex_dump(&self) -> String
	{
		let mut result = String::new();
		let mut byte_index = 0;
		
		while byte_index <= self.bits.len() / 8
		{
			if byte_index % 0x10 == 0
				{ result.push_str(&format!("0x{:04x}: ", byte_index)); }
				
			result.push_str(&format!("0x{:02x} ", self.get_byte(byte_index)));
			byte_index += 1;
			
			if byte_index % 0x10 == 0
				{ result.push_str("\n"); }
		}
		
		result
	}


	pub fn get_min_bit_length(radix: usize, value_str: &str) -> Result<usize, String>
	{
		let mut bit_length = 0;
		match radix
		{
			10 =>
			{
				let mut value = match value_str.parse::<u64>()
				{
					Ok(x) => x,
					Err(_) => return Err(format!("decimal value `{}` is too large", value_str))
				};
				
				while value != 0
				{
					bit_length += 1;
					value >>= 1;
				}
			}
			
			16 => bit_length += value_str.chars().count() * 4,
			2 => bit_length += value_str.chars().count(),
			_ => unreachable!()
		}
		
		Ok(bit_length)
	}
}


impl fmt::Debug for BitVec
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
        write!(f, "{}", self.get_hex_dump())
    }
}


#[cfg(test)]
#[test]
fn test_from_str_sized()
{
	let pass = |bit_size: usize, radix: usize, value_str: &str, expected_bits: Vec<bool>|
		assert_eq!(BitVec::new_from_str_sized(bit_size, radix, value_str).unwrap().get_vec(), &expected_bits);
		
	let fail = |bit_size: usize, radix: usize, value_str: &str|
		assert!(BitVec::new_from_str_sized(bit_size, radix, value_str).is_err());

	pass(0, 10, "0", vec![]);

	pass(1, 10, "0", vec![false]);
	pass(1, 10, "1", vec![true]);
	fail(1, 10, "2");
	
	pass(1, 16, "0", vec![false]);
	pass(1, 16, "00", vec![false]);
	pass(1, 16, "00000000", vec![false]);
	pass(1, 16, "1", vec![true]);
	pass(1, 16, "01", vec![true]);
	pass(1, 16, "00000001", vec![true]);
	fail(1, 16, "2");
	fail(1, 16, "02");
	
	pass(1, 2, "0", vec![false]);
	pass(1, 2, "00", vec![false]);
	pass(1, 2, "00000000", vec![false]);
	pass(1, 2, "1", vec![true]);
	pass(1, 2, "01", vec![true]);
	pass(1, 2, "00000001", vec![true]);
	fail(1, 2, "10");
	
	pass(2, 16, "0", vec![false; 2]);
	pass(3, 16, "0", vec![false; 3]);
	pass(4, 16, "0", vec![false; 4]);
	pass(5, 16, "0", vec![false; 5]);
	pass(6, 16, "0", vec![false; 6]);
	pass(7, 16, "0", vec![false; 7]);
	pass(8, 16, "0", vec![false; 8]);
	pass(16, 16, "0", vec![false; 16]);
	pass(32, 16, "0", vec![false; 32]);
	pass(64, 16, "0", vec![false; 64]);
	pass(128, 16, "0", vec![false; 128]);
	
	pass(2, 16, "03", vec![true; 2]);
	pass(3, 16, "07", vec![true; 3]);
	pass(4, 16, "0f", vec![true; 4]);
	pass(5, 16, "1f", vec![true; 5]);
	pass(6, 16, "3f", vec![true; 6]);
	pass(7, 16, "7f", vec![true; 7]);
	pass(8, 16, "ff", vec![true; 8]);
	pass(8, 16, "FF", vec![true; 8]);
	pass(16, 16, "ffff", vec![true; 16]);
	pass(32, 16, "ffffffff", vec![true; 32]);
	pass(64, 16, "ffffffffffffffff", vec![true; 64]);
	pass(128, 16, "ffffffffffffffffffffffffffffffff", vec![true; 128]);
}