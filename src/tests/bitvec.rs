#![cfg(test)]


use util::bitvec::BitVec;


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