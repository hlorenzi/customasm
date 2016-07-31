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
		
		Ok(bitvec)
	}


	pub fn new_from_str_min(radix: usize, value_str: &str) -> Result<BitVec, String>
	{	
		let mut bitvec = try!(BitVec::new_from_str(radix, value_str));
		bitvec.trim();
		Ok(bitvec)
	}


	pub fn new_from_str_sized(bit_num: usize, radix: usize, value_str: &str) -> Result<BitVec, String>
	{
		let mut bitvec = try!(BitVec::new_from_str_min(radix, value_str));
		
		if bitvec.len() > bit_num
			{ return Err(format!("value `{}` does not fit given size of `{}`", value_str, bit_num)); }
			
		bitvec.zero_extend(bit_num);
		Ok(bitvec)
	}
	
	
	pub fn len(&self) -> usize
	{
		self.bits.len()
	}
	
	
	pub fn compare(&self, other: &BitVec) -> bool
	{
		self.bits == other.bits
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
	
	
	pub fn get_bit_rev(&self, index: usize) -> bool
	{
		if index >= self.bits.len()
			{ false }
		else
			{ self.bits[self.bits.len() - 1 - index] }
	}
	
	
	pub fn slice(&self, left: usize, right: usize) -> BitVec
	{
		let mut result = BitVec::new();
		
		if right < left
		{
			for i in right..(left + 1)
				{ result.insert_bit(0, self.get_bit_rev(i)); }
		}
		else
		{
			for i in left..(right + 1)
				{ result.push_bit(self.get_bit_rev(i)); }
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
		let mut bit_index = 0;
		
		while bit_index < self.bits.len()
		{
			result.push(self.get_byte(bit_index / 8));
			bit_index += 8;
		}
		
		result
	}
	
		
	pub fn get_bin_str(&self) -> String
	{
		let mut result = String::new();
		
		for bit in self.bits.iter()
			{ result.push_str(if *bit { "1" } else { "0" }); }
		
		result
	}
	
		
	pub fn get_bin_dump(&self) -> String
	{
		let mut result = String::new();
		let mut byte_index = 0;
		
		result.push_str("         | ");
		for i in 0..8
			{ result.push_str(&format!("       {:01x} ", i)); }
		
		result.push_str("\n");
		result.push_str("---------+-");
		for _ in 0..8
			{ result.push_str("---------"); }
			
		result.push_str("\n");
		while byte_index <= self.bits.len() / 8
		{
			if byte_index % 0x8 == 0
				{ result.push_str(&format!("{:08x} | ", byte_index)); }
				
			result.push_str(&format!("{:08b} ", self.get_byte(byte_index)));
			byte_index += 1;
			
			if byte_index % 0x8 == 0
				{ result.push_str("\n"); }
		}
		
		result
	}
	
		
	pub fn get_hex_str(&self) -> String
	{
		let mut result = String::new();
		let mut bit_index = 0;
		
		while bit_index < self.bits.len()
		{
			result.push_str(&format!("{:02x}", self.get_byte(bit_index / 8)));
			bit_index += 8;
		}
		
		result
	}
	
		
	pub fn get_hex_dump(&self) -> String
	{
		let mut result = String::new();
		
		result.push_str("         | ");
		for i in 0..16
			{ result.push_str(&format!(" {:01x} ", i)); }
		
		result.push_str("\n");
		result.push_str("---------+-");
		for _ in 0..16
			{ result.push_str("---"); }
			
		result.push_str("\n");
		
		let mut bit_index = 0;
		while bit_index < self.bits.len()
		{
			if bit_index % 0x80 == 0
				{ result.push_str(&format!("{:08x} | ", bit_index / 8)); }
				
			result.push_str(&format!("{:02x} ", self.get_byte(bit_index / 8)));
			bit_index += 8;
			
			if bit_index % 0x80 == 0
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