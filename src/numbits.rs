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


pub fn get_bits(bit_num: usize, radix: usize, value_str: &str) -> Result<Vec<bool>, String>
{
	if bit_num == 0
		{ return Err("invalid bit length".to_string()); }
	
	let mut bits = Vec::new();
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
				bits.insert(0, value & 1 != 0);
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
					bits.push(nibble & 0b1000 != 0);
					nibble <<= 1;
				}
			}
		}
		
		2 =>
		{
			for c in value_str.chars()
				{ bits.push(c == '1'); }
		}
		
		_ => unreachable!()
	}
	
	while bits.len() > 1 && !bits[0]
		{ bits.remove(0); }
	
	if bits.len() > bit_num
		{ return Err(format!("value `{}` does not fit given size of `{}`", value_str, bit_num)); }
		
	while bits.len() < bit_num
		{ bits.insert(0, false); }
	
	Ok(bits)
}


#[cfg(test)]
#[test]
fn test_get_bits()
{
	let pass = |bit_size: usize, radix: usize, value_str: &str, expected_bits: Vec<bool>|
		assert_eq!(get_bits(bit_size, radix, value_str).expect(
			&format!("invalid test: ({}, {}, {})", bit_size, radix, value_str)), expected_bits);
		
	let fail = |bit_size: usize, radix: usize, value_str: &str|
		assert!(get_bits(bit_size, radix, value_str).is_err());

	fail(0, 10, "0");

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