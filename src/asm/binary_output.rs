pub struct BinaryOutput
{
	bits: Vec<bool>
}


impl BinaryOutput
{
	pub fn new() -> BinaryOutput
	{
		BinaryOutput
		{
			bits: Vec::new()
		}
	}
	
	
	pub fn write(&mut self, bit_index: usize, bit: bool)
	{
		while self.bits.len() <= bit_index
			{ self.bits.push(false); }
			
		self.bits[bit_index] = bit;
	}
	
	
	pub fn read(&self, bit_index: usize) -> bool
	{
		if bit_index >= self.bits.len()
			{ false }
		else
			{ self.bits[bit_index] }
	}
	
	
	pub fn len(&self) -> usize
	{
		self.bits.len()
	}
	
	
	pub fn generate_binstr(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_str(1, start_bit, end_bit)
	}
	
	
	pub fn generate_hexstr(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_str(4, start_bit, end_bit)
	}
	
	
	fn generate_str(&self, digit_bits: usize, start_bit: usize, end_bit: usize) -> String
	{
		let mut result = String::new();
		
		let mut index = start_bit;
		while index < end_bit
		{
			let mut digit: u8 = 0;
			for _ in 0..digit_bits
			{
				digit <<= 1;
				digit |= if self.read(index) { 1 } else { 0 };
				index += 1;
			}
			
			let c = if digit < 10
				{ ('0' as u8 + digit) as char }
			else
				{ ('a' as u8 + digit - 10) as char };
				
			result.push(c);
		}
		
		result
	}
}