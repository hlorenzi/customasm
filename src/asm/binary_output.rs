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
	
	
	pub fn generate_str(&self, digit_bits: usize, start_bit: usize, end_bit: usize) -> String
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
	
	
	pub fn generate_bindump(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_dump(1, start_bit, end_bit, 8, 8)
	}
	
	
	pub fn generate_hexdump(&self, start_bit: usize, end_bit: usize) -> String
	{
		self.generate_dump(4, start_bit, end_bit, 8, 16)
	}
	
	
	pub fn generate_dump(&self, digit_bits: usize, start_bit: usize, end_bit: usize, byte_bits: usize, bytes_per_line: usize) -> String
	{
		let mut result = String::new();
		
		let line_start = start_bit / (byte_bits * bytes_per_line);
		let line_end = (end_bit + (bytes_per_line - 1) * byte_bits) / (byte_bits * bytes_per_line);
		
		let addr_max_width = format!("{:x}", (line_end - 1) * bytes_per_line).len();
		
		for line_index in line_start..line_end
		{
			result.push_str(&format!(" {:01$x} | ", line_index * bytes_per_line, addr_max_width));
			
			for byte_index in 0..bytes_per_line
			{
				for digit_index in 0..(byte_bits / digit_bits)
				{
					let digit_first_bit = (line_index * bytes_per_line + byte_index) * byte_bits + digit_index * digit_bits;
					
					if digit_first_bit >= self.len()
					{
						result.push('.');
						continue;
					}
					
					let mut digit = 0;
					for bit_index in 0..digit_bits
					{
						digit <<= 1;
						digit |= if self.read(digit_first_bit + bit_index) { 1 } else { 0 };
					}
			
					let c = if digit < 10
						{ ('0' as u8 + digit) as char }
					else
						{ ('a' as u8 + digit - 10) as char };
					
					result.push(c);
				}
				
				result.push(' ');
			}
			
			result.push_str("| ");
			
			if byte_bits == 8
			{
				for byte_index in 0..bytes_per_line
				{
					let byte_first_bit = (line_index * bytes_per_line + byte_index) * byte_bits;
						
					if byte_first_bit >= self.len()
					{
						result.push('.');
						continue;
					}
						
					let mut byte = 0u8;
					for bit_index in 0..byte_bits
					{
						byte <<= 1;
						byte |= if self.read(byte_first_bit + bit_index) { 1 } else { 0 };
					}
					
					let c = byte as char;
					
					if c == ' ' || c == '\t' || c == '\r' || c == '\n'
						{ result.push(' '); }
					else if c as u8 >= 0x80 || c < ' '
						{ result.push('.'); }
					else
						{ result.push(c); }
				}
				
				result.push_str(" |");
			}
			
			result.push('\n');
		}
		
		result
	}
	
	
	pub fn generate_binary(&self, start_bit: usize, end_bit: usize) -> Vec<u8>
	{
		let mut result = Vec::new();
		
		let mut index = start_bit;
		while index < end_bit
		{
			let mut byte: u8 = 0;
			for _ in 0..8
			{
				byte <<= 1;
				byte |= if self.read(index) { 1 } else { 0 };
				index += 1;
			}
			
			result.push(byte);
		}
		
		result
	}
}