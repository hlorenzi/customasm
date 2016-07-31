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


static DEF_SIMPLE: &'static str =
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


static DEF_TRUNC: &'static str =
"
	.align 8
	.address 8
	
	trunca {a: u16}       -> 8'0x1a a[15:0]
	truncb {a: u16}       -> 8'0x1b a[15:8] a[7:0]
	truncc {a: u16}       -> 8'0x1c a[15:8]
	truncd {a: u16}       -> 8'0x1d a[7:0]
	trunce {a: u16}       -> 8'0x1e a[7:0] a[15:8]
	truncf {a: u16}       -> 8'0x1f a[0:15]
	
	trunc0 {a: u16}       -> 8'0x10 a[31:0]
";


#[test]
fn test_instructions_simple()
{	
	pass("", "", "");
	pass(DEF_SIMPLE, "", "");
	pass(DEF_SIMPLE, "halt", "aa");
	pass(DEF_SIMPLE, "add 0x5", "bb05");
	pass(DEF_SIMPLE, "add 0x56", "bb56");
	pass(DEF_SIMPLE, "add 0x567", "cc0567");
	pass(DEF_SIMPLE, "add 0x5678", "cc5678");
	pass(DEF_SIMPLE, "sub 0x1 0x5", "dd0105");
	pass(DEF_SIMPLE, "sub 0x12 0x56", "dd1256");
	pass(DEF_SIMPLE, "sub 0x12 0x567", "ee00120567");
	pass(DEF_SIMPLE, "sub 0x12 0x5678", "ee00125678");
	pass(DEF_SIMPLE, "sub 0x123 0x56", "ee01230056");
	pass(DEF_SIMPLE, "sub 0x123 0x567", "ee01230567");
	pass(DEF_SIMPLE, "sub 0x1234 0x5678", "ee12345678");
	
	pass(DEF_SIMPLE, "halt \n halt", "aaaa");
}


#[test]
fn test_instructions_arg_slice()
{
	pass(DEF_TRUNC, "trunca 0x1234", "1a1234");
	pass(DEF_TRUNC, "truncb 0x1234", "1b1234");
	pass(DEF_TRUNC, "truncc 0x1234", "1c12");
	pass(DEF_TRUNC, "truncd 0x1234", "1d34");
	pass(DEF_TRUNC, "trunce 0x1234", "1e3412");
	pass(DEF_TRUNC, "truncf 0x1234", "1f2c48");
	pass(DEF_TRUNC, "trunc0 0x1234", "1000001234");
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
	pass(DEF_SIMPLE, "start: \n jmp start", "ff00");
	
	pass(DEF_SIMPLE, "jmp loop \n loop: \n halt", "ff02aa");
	pass(DEF_SIMPLE, "jmp loop \n loop: \n jmp loop", "ff02ff02");
	
	pass(DEF_SIMPLE, "start: \n 'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", "ff00ff02");
	pass(DEF_SIMPLE, "          'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", "ff00ff02");
}