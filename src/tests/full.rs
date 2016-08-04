#![cfg(test)]


use definition;
use assembler;
use util::bitvec::BitVec;


fn pass(def_str: &str, asm_str: &str, expected_out_radix: usize, expected_out: &str)
{
	let def = definition::parse("test", &def_str.chars().collect::<Vec<char>>()).unwrap();
	let out = assembler::assemble(&def, "test", &asm_str.chars().collect::<Vec<char>>()).unwrap();
	assert!(out.compare(&BitVec::new_from_str(expected_out_radix, expected_out).unwrap()),
		format!("\ntest failed:\n\ndef:\n{}\n\nasm:\n{}\n\nexpected: {}\n     got: {}\n", def_str, asm_str, expected_out, out.get_hex_str()));
}


static DEF_SIMPLE: &'static str =
"
	.align 8
	.address 8
	
	halt                  -> 8'0xaa
	add {a: u8}           -> 8'0xbb a[ 7:0]
	add {a: u16}          -> 8'0xcc a[15:0]
	sub {a: u8}  {b: u8}  -> 8'0xdd a[ 7:0] b[ 7:0]
	sub {a: u16} {b: u16} -> 8'0xee a[15:0] b[15:0]
	jmp {a: u8}           -> 8'0xff a[ 7:0]
";


static DEF_TRUNC: &'static str =
"
	.align 8
	.address 8
	
	trunca {a: u16} -> 8'0x1a a[15:0]
	truncb {a: u16} -> 8'0x1b a[15:8] a[ 7:0]
	truncc {a: u16} -> 8'0x1c a[15:8]
	truncd {a: u16} -> 8'0x1d a[ 7:0]
	trunce {a: u16} -> 8'0x1e a[ 7:0] a[15:8]
	truncf {a: u16} -> 8'0x1f a[0:15]
	
	trunc0 {a: u16} -> 8'0x10 a[31:0]
";


#[test]
fn test_instructions_simple()
{	
	pass("", "", 16, "");
	pass(DEF_SIMPLE, "", 16, "");
	pass(DEF_SIMPLE, "halt", 16, "aa");
	pass(DEF_SIMPLE, "add 0x5", 16, "bb05");
	pass(DEF_SIMPLE, "add 0x56", 16, "bb56");
	pass(DEF_SIMPLE, "add 0x567", 16, "cc0567");
	pass(DEF_SIMPLE, "add 0x5678", 16, "cc5678");
	pass(DEF_SIMPLE, "sub 0x1 0x5", 16, "dd0105");
	pass(DEF_SIMPLE, "sub 0x12 0x56", 16, "dd1256");
	pass(DEF_SIMPLE, "sub 0x12 0x567", 16, "ee00120567");
	pass(DEF_SIMPLE, "sub 0x12 0x5678", 16, "ee00125678");
	pass(DEF_SIMPLE, "sub 0x123 0x56", 16, "ee01230056");
	pass(DEF_SIMPLE, "sub 0x123 0x567", 16, "ee01230567");
	pass(DEF_SIMPLE, "sub 0x1234 0x5678", 16, "ee12345678");
	
	pass(DEF_SIMPLE, "halt \n halt", 16, "aaaa");
}


#[test]
fn test_instructions_arg_slice()
{
	pass(DEF_TRUNC, "trunca 0x1234", 16, "1a1234");
	pass(DEF_TRUNC, "truncb 0x1234", 16, "1b1234");
	pass(DEF_TRUNC, "truncc 0x1234", 16, "1c12");
	pass(DEF_TRUNC, "truncd 0x1234", 16, "1d34");
	pass(DEF_TRUNC, "trunce 0x1234", 16, "1e3412");
	pass(DEF_TRUNC, "truncf 0x1234", 16, "1f2c48");
	pass(DEF_TRUNC, "trunc0 0x1234", 16, "1000001234");
}


#[test]
fn test_literals_simple()
{
	pass(".align 1", ".d1 1, 0, 1, 0", 2, "1010");
	pass(".align 1", ".d1 1, 0, 1, 0", 16, "a");
	pass(".align 1", ".d1 0, 1, 0, 1, 0, 1, 0, 1", 16, "55");
	pass(".align 2", ".d2 2, 3", 2, "1011");
	pass(".align 2", ".d2 2, 3", 16, "b");
	pass(".align 2", ".d2 2, 3, 1, 0", 16, "b4");
	pass(".align 2", ".d8 0xb4", 16, "b4");
	pass(".align 3", ".d3 0b101", 2, "101");
	pass(".align 3", ".d3 0b101, 0b110", 2, "101110");
	pass(".align 3", ".d3 0b101, 0b110, 0b111, 0b10", 16, "bba");
	pass(".align 4", ".d4 0b1011", 2, "1011");
	pass(".align 5", ".d5 0b10110", 2, "10110");
	pass(".align 6", ".d6 0b101100", 2, "101100");
	pass(".align 7", ".d7 0b1011001", 2, "1011001");
	
	pass(".align 8", ".d8 0xab, 0xcd, 0xef", 16, "abcdef");
	pass(".align 8", ".d16 0xabcd, 0xcdef, 0xefab", 16, "abcdcdefefab");
	pass(".align 8", ".d32 0x12345678, 0x1, 0xabcdef", 16, "123456780000000100abcdef");
	pass(".align 8", ".d64 0x12345678abcdef00, 0x123", 16, "12345678abcdef000000000000000123");
	pass(".align 8", ".d128 0x12345678abcdef", 16, "00000000000000000012345678abcdef");
}


#[test]
fn test_literals_with_variables()
{
	pass(".align 8", "start: \n .d8 start", 16, "00");
	pass(".align 8", "start: \n .d8 0x12, 0x34, start", 16, "123400");
	pass(".align 8", ".d8 start \n start:", 16, "01");
	pass(".align 8", ".d8 0x12, 0x34, start \n start:", 16, "123403");
	
	pass(".align 8", "start: \n .d8 start, end \n end:", 16, "0002");
	pass(".align 8", "start: \n .d8 end, start \n end:", 16, "0200");
	pass(".align 8", "start: \n .d8 start, 0x45, end \n end:", 16, "004503");
	pass(".align 8", "start: \n .d8 end, 0x45, start \n end:", 16, "034500");
	
	pass(".align 8", ".address 0x1234 \n start: \n .d8 start", 16, "34");
	pass(".align 8", ".address 0x1234 \n start: \n .d16 start", 16, "1234");
	pass(".align 8", ".d8 start  \n .address 0x1234 \n start:", 16, "34");
	pass(".align 8", ".d16 start \n .address 0x1234 \n start:", 16, "1234");
}


#[test]
fn test_labels_simple()
{	
	pass(DEF_SIMPLE, "start: \n jmp start", 16, "ff00");
	
	pass(DEF_SIMPLE, "jmp loop \n loop: \n halt", 16, "ff02aa");
	pass(DEF_SIMPLE, "jmp loop \n loop: \n jmp loop", 16, "ff02ff02");
	
	pass(DEF_SIMPLE, "start: \n 'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", 16, "ff00ff02");
	pass(DEF_SIMPLE, "          'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", 16, "ff00ff02");
}