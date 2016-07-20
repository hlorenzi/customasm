#[cfg(test)]
mod parser_tests
{
	use parser::*;
	
	
	#[test]
	fn test_get_usize()
	{
		let pass = |src: &str, expected: usize|
			assert_eq!(Parser::new(&mut src.chars()).get_usize().unwrap(), expected);
			
		let fail = |src: &str|
			assert!(Parser::new(&mut src.chars()).get_usize().is_err());
			
		fail("x");
	
		pass("0", 0);
		pass("1", 1);
		pass("2", 2);
		pass("10", 10);
		fail("1a");
		
		pass("0x0", 0);
		pass("0x1", 1);
		pass("0x2", 2);
		pass("0xa", 10);
		pass("0xf", 15);
		pass("0x10", 16);
		pass("0xff", 255);
		pass("0xffffffff", u32::max_value() as usize);
		
		pass("0b0", 0);
		pass("0b1", 1);
		pass("0b10", 2);
		pass("0b1010", 10);
		pass("0b1111", 15);
		pass("0b10000", 16);
		pass("0b11111111", 255);
		pass("0b11111111111111111111111111111111", u32::max_value() as usize);
		fail("0b12");
	}
	
	
	#[test]
	fn test_get_bits()
	{
		let pass = |src: &str, expected_bits: Vec<bool>|
			assert_eq!(Parser::new(&mut src.chars()).get_bits().unwrap(), expected_bits);
			
		let fail = |src: &str|
			assert!(Parser::new(&mut src.chars()).get_bits().is_err());
	
		fail("x");
		fail("0'0");
	
		pass("1'0", vec![false]);
		pass("1'1", vec![true]);
		fail("1'2");
		
		pass("1'0x0", vec![false]);
		pass("1'0x00", vec![false]);
		pass("1'0x00000000", vec![false]);
		pass("1'0x1", vec![true]);
		pass("1'0x01", vec![true]);
		pass("1'0x00000001", vec![true]);
		fail("1'0x2");
		fail("1'0x02");
		fail("1'9999999999999999");
		
		pass("1'0b0", vec![false]);
		pass("1'0b00", vec![false]);
		pass("1'0b00000000", vec![false]);
		pass("1'0b1", vec![true]);
		pass("1'0b01", vec![true]);
		pass("1'0b00000001", vec![true]);
		fail("1'0b10");
		
		pass("2'0x0", vec![false; 2]);
		pass("3'0x0", vec![false; 3]);
		pass("4'0x0", vec![false; 4]);
		pass("5'0x0", vec![false; 5]);
		pass("6'0x0", vec![false; 6]);
		pass("7'0x0", vec![false; 7]);
		pass("8'0x0", vec![false; 8]);
		pass("16'0x0", vec![false; 16]);
		pass("32'0x0", vec![false; 32]);
		pass("64'0x0", vec![false; 64]);
		pass("128'0x0", vec![false; 128]);
		
		pass("2'0x03", vec![true; 2]);
		pass("3'0x07", vec![true; 3]);
		pass("4'0x0f", vec![true; 4]);
		pass("5'0x1f", vec![true; 5]);
		pass("6'0x3f", vec![true; 6]);
		pass("7'0x7f", vec![true; 7]);
		pass("8'0xff", vec![true; 8]);
		pass("16'0xffff", vec![true; 16]);
		pass("32'0xffffffff", vec![true; 32]);
		pass("64'0xffffffffffffffff", vec![true; 64]);
		pass("128'0xffffffffffffffffffffffffffffffff", vec![true; 128]);
		
		pass("0x1'0", vec![false; 1]);
		pass("0x2'0", vec![false; 2]);
		pass("0x10'0", vec![false; 16]);
		pass("0b1'0", vec![false; 1]);
		pass("0b10'0", vec![false; 2]);
		pass("0b10000'0", vec![false; 16]);
	}
}