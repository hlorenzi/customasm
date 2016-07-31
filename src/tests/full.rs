#![cfg(test)]


use definition;
use assembler;
use util::bitvec::BitVec;


fn pass(def_str: &str, asm_str: &str, expected_out: &str)
{
	let def = definition::parse(&def_str.chars().collect::<Vec<char>>()).unwrap();
	let out = assembler::assemble(&def, &asm_str.chars().collect::<Vec<char>>()).unwrap();
	assert!(out.compare(&BitVec::new_from_str(16, expected_out).unwrap()),
		format!("\ntest failed:\n\ndef:\n{}\n\nasm:\n{}\n\nexpected: {}\n     got: {}\n", def_str, asm_str, expected_out, out.get_hex_str()));
}


static DEF1: &'static str =
"
	.align 8
	.address 8
	
	halt                  -> 8'0xaa
	add {a: u8}           -> 8'0xbb a
	add {a: u16}          -> 8'0xcc a
	sub {a: u8}  {b: u8}  -> 8'0xdd a b
	sub {a: u16} {b: u16} -> 8'0xee a b
	jmp {a: u8}           -> 8'0xff a
";


#[test]
fn test_instructions_simple()
{	
	pass("", "", "");
	pass(DEF1, "", "");
	pass(DEF1, "halt", "aa");
	pass(DEF1, "add 0x5", "bb05");
	pass(DEF1, "add 0x56", "bb56");
	pass(DEF1, "add 0x567", "cc0567");
	pass(DEF1, "add 0x5678", "cc5678");
	pass(DEF1, "sub 0x1 0x5", "dd0105");
	pass(DEF1, "sub 0x12 0x56", "dd1256");
	pass(DEF1, "sub 0x12 0x567", "ee00120567");
	pass(DEF1, "sub 0x12 0x5678", "ee00125678");
	pass(DEF1, "sub 0x123 0x56", "ee01230056");
	pass(DEF1, "sub 0x123 0x567", "ee01230567");
	pass(DEF1, "sub 0x1234 0x5678", "ee12345678");
	
	pass(DEF1, "halt \n halt", "aaaa");
}


#[test]
fn test_literals_simple()
{
	pass(".align 1", ".d1 1, 0, 1, 0", "a");
	pass(".align 1", ".d1 0, 1, 0, 1, 0, 1, 0, 1", "55");
	pass(".align 2", ".d2 2, 3", "b");
	pass(".align 2", ".d2 2, 3, 1, 0", "b4");
	pass(".align 2", ".d8 0xb4", "b4");
	pass(".align 3", ".d3 0b101, 0b110, 0b111, 0b10", "bba");
	
	pass(".align 8", ".d8 0xab, 0xcd, 0xef", "abcdef");
	pass(".align 8", ".d16 0xabcd, 0xcdef, 0xefab", "abcdcdefefab");
	pass(".align 8", ".d32 0x12345678, 0x1, 0xabcdef", "123456780000000100abcdef");
	pass(".align 8", ".d64 0x12345678abcdef00, 0x123", "12345678abcdef000000000000000123");
	pass(".align 8", ".d128 0x12345678abcdef", "00000000000000000012345678abcdef");
}


#[test]
fn test_labels_simple()
{	
	pass(DEF1, "start: \n jmp start", "ff00");
	
	pass(DEF1, "jmp loop \n loop: \n halt", "ff02aa");
	pass(DEF1, "jmp loop \n loop: \n jmp loop", "ff02ff02");
	
	pass(DEF1, "start: \n 'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", "ff00ff02");
	pass(DEF1, "          'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", "ff00ff02");
}