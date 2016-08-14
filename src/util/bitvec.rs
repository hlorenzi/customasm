use std::fmt;
use util::integer::Integer;


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
	
	
	pub fn new_from_bytes(bytes: &[u8]) -> BitVec
	{
		let mut bitvec = BitVec::new();
		
		for mut byte in bytes.iter().cloned()
		{
			for _ in 0..8
			{
				bitvec.push_bit(byte & 0x80 != 0);
				byte <<= 1;
			}
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
	
	
	pub fn to_i64(&self) -> Option<i64>
	{
		let mut result: i64 = 0;
		
		for bit in 0..self.bits.len()
		{
			result = match result.checked_shl(1)
			{
				Some(result) => result,
				None => return None
			};
			
			result |= if self.bits[bit] { 1 } else { 0 };
		}
		
		Some(result)
	}
	
	
	pub fn len(&self) -> usize
	{
		self.bits.len()
	}
	
	
	pub fn compare(&self, other: &BitVec) -> bool
	{
		self.bits == other.bits
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
	
	
	pub fn set(&mut self, index: usize, value: &Integer)
	{
		for i in 0..value.get_width()
			{ self.set_bit(index + i, value.get_bit(value.get_width() - 1 - i)); }
	}
	
	
	pub fn set_bitvec(&mut self, index: usize, bitvec: &BitVec)
	{
		for i in 0..bitvec.len()
			{ self.set_bit(index + i, bitvec.get_bit(i)); }
	}
	
	
	pub fn push(&mut self, value: &Integer)
	{
		for i in 0..value.get_width()
			{ self.push_bit(value.get_bit(value.get_width() - 1 - i)); }
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
		
		result.push_str("          | ");
		for i in 0..8
			{ result.push_str(&format!("       {:01x} ", i)); }
		
		result.push_str("|\n");
		result.push_str("----------+-");
		for _ in 0..8
			{ result.push_str("---------"); }
		
		result.push_str("+--");
		for _ in 0..8
			{ result.push_str("-"); }
		
		result.push_str("\n");
		
		let mut byte_index = 0;
		while byte_index * 8 < self.bits.len()
		{
			result.push_str(&format!(" {:08x} | ", byte_index));
			
			for i in 0..8
			{
				for j in 0..8
				{
					if (byte_index + i) * 8 + j >= self.bits.len()
						{ result.push('.'); }
					else
						{ result.push_str(&format!("{}", if self.get_bit((byte_index + i) * 8 + j) { "1" } else { "0" })); }
				}
				
				result.push_str(" ");
			}
			
			result.push_str("| ");
			
			for i in 0..8
			{
				if (byte_index + i) * 8 >= self.bits.len()
					{ result.push('.'); }
				else
					{ result.push(get_byte_repr(self.get_byte(byte_index + i))); }
			}
			
			result.push_str("\n");
			byte_index += 8;
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
		
		result.push_str("          | ");
		for i in 0..16
			{ result.push_str(&format!(" {:01x} ", i)); }
		
		result.push_str("|\n----------+-");
		for _ in 0..16
			{ result.push_str("---"); }
		
		result.push_str("+--");
		for _ in 0..16
			{ result.push_str("-"); }
			
		result.push_str("\n");
		
		let mut byte_index = 0;
		while byte_index * 8 < self.bits.len()
		{
			result.push_str(&format!(" {:08x} | ", byte_index));
			
			for i in 0..16
			{
				if (byte_index + i) * 8 >= self.bits.len()
					{ result.push_str(".. "); }
				else
					{ result.push_str(&format!("{:02x} ", self.get_byte(byte_index + i))); }
			}
			
			result.push_str("| ");
			
			for i in 0..16
			{
				if (byte_index + i) * 8 >= self.bits.len()
					{ result.push('.'); }
				else
					{ result.push(get_byte_repr(self.get_byte(byte_index + i))); }
			}
			
			result.push_str("\n");
			byte_index += 16;
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


fn get_byte_repr(byte: u8) -> char
{
	if byte >= 0x20 && byte <= 0x7e
		{ byte as char }
	else if byte as char == '\n' || byte as char == '\r' || byte as char == '\t'
		{ ' ' }
	else
		{ '.' }
}