#![cfg(test)]


use assembler::Assembler;
use definition::Definition;
use util::bitvec::BitVec;
use util::filehandler::{FileHandler, CustomFileHandler};
use std::path::Path;


fn pass(
	def_str: &str,
	asm_str: &str,
	expected_out_radix: usize,
	expected_out: &str)
{
	let mut filehandler = CustomFileHandler::new();
	filehandler.add("test", asm_str);
	
	pass_filehandler(def_str, &filehandler, "test", expected_out_radix, expected_out);
}


fn pass_filehandler(
	def_str: &str,
	filehandler: &FileHandler,
	main_filename: &str,
	expected_out_radix: usize,
	expected_out: &str)
{
	let def = Definition::from_str(def_str).unwrap();
	let out = Assembler::assemble_file(&def, filehandler, &Path::new(main_filename)).unwrap();
	
	if !out.compare(&BitVec::new_from_str(expected_out_radix, expected_out).unwrap())
	{
		println!("expected: {}", expected_out);
		println!("     got: {}", out.get_hex_str());
		panic!("full test failed but expected to pass");
	}
}


fn fail(
	def_str: &str,
	asm_str: &str,
	expected_error_line: usize,
	expected_error_substr: &str)
{
	let mut filehandler = CustomFileHandler::new();
	filehandler.add("test", asm_str);
	
	fail_filehandler(def_str, &filehandler, "test", "test", expected_error_line, expected_error_substr);
}


fn fail_filehandler(
	def_str: &str,
	filehandler: &FileHandler,
	main_filename: &str,
	expected_error_file: &str,
	expected_error_line: usize,
	expected_error_substr: &str)
{
	let def = Definition::from_str(def_str).unwrap();
	match Assembler::assemble_file(&def, filehandler, &Path::new(main_filename))
	{
		Ok(_) => panic!("full test passed but error expected"),
			
		Err(err) =>
			if !err.file_is(expected_error_file) ||
				!err.line_is(expected_error_line) ||
				!err.contains_str(expected_error_substr)
			{
				println!(" expected error msg: {}", expected_error_substr);
				println!("      got error msg: {}", err.get_msg());
				println!("expected error file: {}", expected_error_file);
				println!("     got error file: {}", err.get_file());
				println!("expected error line: {}", expected_error_line);
				println!("     got error line: {}", err.get_line());
				panic!("full test error mismatch");
			}
	}
}


static DEF_SIMPLE: &'static str =
"
	#align 8
	
	halt        -> 8'0x10
	add {a}     -> 8'0x11 a[7:0]
	sub {a} {b} -> 8'0x12 a[7:0] b[7:0]
	jmp {a}     -> 8'0x13 a[7:0]
";


static DEF_WHITESPACE: &'static str =
"

	#align 8 ; comment
	
	; comment
	
	halt        -> 8  ' 0x10 ; comment
	; comment
	
	add {a}     -> 8'0x11 a [ 7 : 0 ] ; comment
	sub {a} {b} -> 8'0x12 a[7:0] ; comment b[7:0]
	
	jmp { a }   -> 8'0x13 a[7:0]
	;xyz {a}    -> 8'0x14 a[7:0]
	
";


static DEF_CONSTRAINT: &'static str =
"
	#align 8
	
	simple0 {a! : _ <= 0xff}       -> 8'0x00 a[ 7:0]
	simple0 {a! : _ <= 0xffff}     -> 8'0x01 a[15:0]
	simple0 {a! : _ <= 0xffffff}   -> 8'0x02 a[23:0]
	simple0 {a  : _ <= 0xffffffff} -> 8'0x03 a[31:0]
	
	multi0 {a! : _ <=       0xff} {b! : _ <=       0xff} -> 8'0x20 a[ 7:0] b[ 7:0]
	multi0 {a! : _ <=       0xff} {b! : _ <=     0xffff} -> 8'0x21 a[ 7:0] b[15:0]
	multi0 {a! : _ <=     0xffff} {b! : _ <=       0xff} -> 8'0x22 a[15:0] b[ 7:0]
	multi0 {a! : _ <=     0xffff} {b! : _ <=     0xffff} -> 8'0x23 a[15:0] b[15:0]
	multi0 {a  : _ <= 0xffffffff} {b  : _ <= 0xffffffff} -> 8'0x24 a[31:0] b[31:0]
	
	pc0 {a: _ + pc <= 0xff} -> 8'0x30 (a + pc)[7:0]
	
	range0 {a: _ >= 0x80 && _ <= 0x90} -> 8'0x40 a[7:0]
	range1 {a: _ <  0x80 || _ >  0x90} -> 8'0x41 a[7:0]
";


static DEF_EXPR: &'static str =
"
	#align 8
	
	slice0 {a} -> 8'0x10 a[15:0]
	slice1 {a} -> 8'0x11 a[15:8] a[ 7:0]
	slice2 {a} -> 8'0x12 a[15:8]
	slice3 {a} -> 8'0x13 a[ 7:0]
	slice4 {a} -> 8'0x14 a[ 7:0] a[15:8]
	slice5 {a} -> 8'0x15 a[31:0]
	
	expr0 {a}  -> 8'0x20 (a + 1)[7:0]
	expr1 {a}  -> 8'0x21 (a + a)[7:0]
	expr2 {a}  -> 8'0x22 (a * a)[7:0]
	
	pc0        -> 8'0x30 pc[7:0]
	pc1 {a}    -> 8'0x31 (pc + a)[7:0]
";


#[test]
fn test_rules_simple()
{	
	pass("", "", 16, "");
	pass(DEF_SIMPLE, "", 16, "");
	pass(DEF_SIMPLE, "halt", 16, "10");
	pass(DEF_SIMPLE, "add 0x5", 16, "1105");
	pass(DEF_SIMPLE, "add 0x56", 16, "1156");
	pass(DEF_SIMPLE, "sub 0x1 0x5", 16, "120105");
	pass(DEF_SIMPLE, "sub 0x12 0x56", 16, "121256");
	
	pass(DEF_SIMPLE, "halt \n halt", 16, "1010");
	
	fail(DEF_SIMPLE, "xyz", 1, "no match");
	fail(DEF_SIMPLE, "halt \n xyz", 2, "no match");
	fail(DEF_SIMPLE, "add", 1, "no match");
	fail(DEF_SIMPLE, "sub 0x1", 1, "no match");
}


#[test]
fn test_whitespace()
{
	pass(DEF_WHITESPACE, "", 16, "");
	pass(DEF_WHITESPACE, "halt", 16, "10");
	pass(DEF_WHITESPACE, "add 0x5", 16, "1105");
	pass(DEF_WHITESPACE, "add 0x56", 16, "1156");
	pass(DEF_WHITESPACE, "sub 0x1 0x5", 16, "1201");
	pass(DEF_WHITESPACE, "sub 0x12 0x56", 16, "1212");
	
	pass(DEF_WHITESPACE, "halt \n halt", 16, "1010");
	
	pass(DEF_WHITESPACE, "", 16, "");
	pass(DEF_WHITESPACE, "halt ; comment", 16, "10");
	pass(DEF_WHITESPACE, "add 0x5 ; comment", 16, "1105");
	pass(DEF_WHITESPACE, "add 0x56 ; comment", 16, "1156");
	pass(DEF_WHITESPACE, "sub 0x1 0x5 ; comment", 16, "1201");
	pass(DEF_WHITESPACE, "sub 0x12 0x56 ; comment", 16, "1212");
	
	pass(DEF_WHITESPACE, "   ; comment halt           \n    halt", 16, "10");
	pass(DEF_WHITESPACE, "\n           halt           \n    halt", 16, "1010");
	pass(DEF_WHITESPACE, "             halt ; comment \n    halt", 16, "1010");
	pass(DEF_WHITESPACE, "             halt           \n \n halt", 16, "1010");
	
	fail(DEF_WHITESPACE, "xyz", 1, "no match");
	fail(DEF_WHITESPACE, "halt \n xyz", 2, "no match");
	fail(DEF_WHITESPACE, "add ; 0x5 comment", 1, "no match");
}


#[test]
fn test_rules_constraints()
{	
	pass(DEF_CONSTRAINT, "", 16, "");
	
	pass(DEF_CONSTRAINT, "simple0 0x1", 16, "0001");
	pass(DEF_CONSTRAINT, "simple0 0xff", 16, "00ff");
	pass(DEF_CONSTRAINT, "simple0 0x100", 16, "010100");
	pass(DEF_CONSTRAINT, "simple0 0xffff", 16, "01ffff");
	pass(DEF_CONSTRAINT, "simple0 0x10000", 16, "02010000");
	pass(DEF_CONSTRAINT, "simple0 0xffffff", 16, "02ffffff");
	pass(DEF_CONSTRAINT, "simple0 0x1000000", 16, "0301000000");
	pass(DEF_CONSTRAINT, "simple0 0xffffffff", 16, "03ffffffff");
	
	pass(DEF_CONSTRAINT, "start: \n simple0 start", 16, "0000");
	pass(DEF_CONSTRAINT, "simple0 start \n start:", 16, "0300000005");
	
	pass(DEF_CONSTRAINT, "multi0 0xff 0xff", 16, "20ffff");
	pass(DEF_CONSTRAINT, "multi0 0xff 0xffff", 16, "21ffffff");
	pass(DEF_CONSTRAINT, "multi0 0xffff 0xff", 16, "22ffffff");
	pass(DEF_CONSTRAINT, "multi0 0xffff 0xffff", 16, "23ffffffff");
	pass(DEF_CONSTRAINT, "multi0 0x123456 0x7890ab", 16, "2400123456007890ab");
	
	pass(DEF_CONSTRAINT, "start: \n multi0   0x1 start", 16, "200100");
	pass(DEF_CONSTRAINT, "start: \n multi0 start   0x1", 16, "200001");
	pass(DEF_CONSTRAINT, "start: \n multi0 start start", 16, "200000");
	pass(DEF_CONSTRAINT, "multi0   0x1 start \n start:", 16, "240000000100000009");
	pass(DEF_CONSTRAINT, "multi0 start   0x1 \n start:", 16, "240000000900000001");
	pass(DEF_CONSTRAINT, "multi0 start start \n start:", 16, "240000000900000009");
	
	pass(DEF_CONSTRAINT, "pc0 0xff", 16, "30ff");
	
	pass(DEF_CONSTRAINT, "range0 0x80", 16, "4080");
	pass(DEF_CONSTRAINT, "range0 0x88", 16, "4088");
	pass(DEF_CONSTRAINT, "range0 0x90", 16, "4090");
	pass(DEF_CONSTRAINT, "range1 0x60", 16, "4160");
	pass(DEF_CONSTRAINT, "range1 0x7f", 16, "417f");
	pass(DEF_CONSTRAINT, "range1 0x91", 16, "4191");
	pass(DEF_CONSTRAINT, "range1 0xa0", 16, "41a0");
	
	fail(DEF_CONSTRAINT, "simple0 start      \n #address 0x100110011 \n start:", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "multi0 start start \n #address 0x100110011 \n start:", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "#d8 0xdd \n pc0 0xff", 2, "not satisfied");
	fail(DEF_CONSTRAINT, "range0 0x70", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "range0 0x7f", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "range0 0x91", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "range0 0xa0", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "range1 0x80", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "range1 0x88", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "range1 0x90", 1, "not satisfied");
}


#[test]
fn test_rules_with_production_expressions()
{
	pass(DEF_EXPR, "", 16, "");
	
	pass(DEF_EXPR, "slice0 0x1234", 16, "101234");
	pass(DEF_EXPR, "slice1 0x1234", 16, "111234");
	pass(DEF_EXPR, "slice2 0x1234", 16, "1212");
	pass(DEF_EXPR, "slice3 0x1234", 16, "1334");
	pass(DEF_EXPR, "slice4 0x1234", 16, "143412");
	pass(DEF_EXPR, "slice5 0x1234", 16, "1500001234");

	pass(DEF_EXPR, "expr0 0x08", 16, "2009");
	pass(DEF_EXPR, "expr1 0x08", 16, "2110");
	pass(DEF_EXPR, "expr2 0x08", 16, "2240");
	
	pass(DEF_EXPR, "pc0", 16, "3000");
	pass(DEF_EXPR, "pc1 0x08", 16, "3108");
	pass(DEF_EXPR, "#d8 0xff \n pc0", 16, "ff3001");
	pass(DEF_EXPR, "#d8 0xff \n pc1 0x08", 16, "ff3109");
}


#[test]
fn test_rules_with_argument_expressions()
{
	pass(DEF_SIMPLE, "add 2 + 3", 16, "1105");
	pass(DEF_SIMPLE, "add 0x50 + 0x06", 16, "1156");
	pass(DEF_SIMPLE, "sub 3 - 2 12 - 7", 16, "120105");
	pass(DEF_SIMPLE, "sub (0x14 - 0x2) 0x58 - 0x2", 16, "121256");
	
	pass(DEF_CONSTRAINT, "simple0 0x100 - 0xff", 16, "0001");
	pass(DEF_CONSTRAINT, "simple0 0x100 - 1", 16, "00ff");
	pass(DEF_CONSTRAINT, "simple0 0xff + 1", 16, "010100");
	pass(DEF_CONSTRAINT, "simple0 0x10000 - 1", 16, "01ffff");
	pass(DEF_CONSTRAINT, "simple0 0xffff + 1", 16, "02010000");
	pass(DEF_CONSTRAINT, "simple0 0x1000000 - 1", 16, "02ffffff");
	pass(DEF_CONSTRAINT, "simple0 0xffffff + 1", 16, "0301000000");
	pass(DEF_CONSTRAINT, "simple0 0x100000000 - 1", 16, "03ffffffff");
	
	pass(DEF_CONSTRAINT, "simple0 start + 1 \n start:", 16, "0300000006");
}


#[test]
fn test_address_directive()
{
	pass(DEF_SIMPLE, "#address 0x80        \n a:    \n jmp a", 16, "1380");
	pass(DEF_SIMPLE, "#address 0x80        \n jmp a \n a:",    16, "1382");
	pass(DEF_SIMPLE, "#address 0x40 + 0x40 \n jmp a \n a:",    16, "1382");

	pass(DEF_SIMPLE, "#address pc        \n a: \n jmp a", 16, "1300");
	pass(DEF_SIMPLE, "#address pc + 0x80 \n a: \n jmp a", 16, "1380");
	
	pass(DEF_SIMPLE, "#address pc + 0x80 \n jmp a \n #address pc + 0x10 \n a:", 16, "1392");
	
	fail(DEF_SIMPLE, "#address  0 - 0x80 \n a: \n jmp a", 1, "invalid");
	fail(DEF_SIMPLE, "#address pc - 0x80 \n a: \n jmp a", 1, "invalid");
}


#[test]
fn test_output_directive()
{
	pass(DEF_SIMPLE, "#output 0x00        \n a:    \n jmp a", 16, "1300");
	pass(DEF_SIMPLE, "#output 0x01        \n a:    \n jmp a", 16, "001300");
	pass(DEF_SIMPLE, "#output 0x01 + 0x01 \n jmp a \n a:",    16, "00001302");

	pass(DEF_SIMPLE, "#output pc                  \n a:    \n jmp a", 16, "1300");
	pass(DEF_SIMPLE, "#address 0x02 \n #output pc \n a:    \n jmp a", 16, "00001302");
	pass(DEF_SIMPLE, "#address 0x02 \n #output pc \n jmp a \n a:",    16, "00001304");
	
	fail(DEF_SIMPLE, "#output  0 - 0x80 \n a: \n jmp a", 1, "invalid");
	fail(DEF_SIMPLE, "#output pc - 0x80 \n a: \n jmp a", 1, "invalid");
}


#[test]
fn test_data_directive_simple()
{
	pass("#align 1", "#d1 1, 0, 1, 0", 2, "1010");
	pass("#align 1", "#d1 1, 0, 1, 0", 16, "a");
	pass("#align 1", "#d1 0, 1, 0, 1, 0, 1, 0, 1", 16, "55");
	pass("#align 2", "#d2 2, 3", 2, "1011");
	pass("#align 2", "#d2 2, 3", 16, "b");
	pass("#align 2", "#d2 2, 3, 1, 0", 16, "b4");
	pass("#align 2", "#d8 0xb4", 16, "b4");
	pass("#align 3", "#d3 0b101", 2, "101");
	pass("#align 3", "#d3 0b101, 0b110", 2, "101110");
	pass("#align 3", "#d3 0b101, 0b110, 0b111, 0b10", 16, "bba");
	pass("#align 4", "#d4 0b1011", 2, "1011");
	pass("#align 5", "#d5 0b10110", 2, "10110");
	pass("#align 6", "#d6 0b101100", 2, "101100");
	pass("#align 7", "#d7 0b1011001", 2, "1011001");
	
	pass("#align 8", "#d8 0xab, 0xcd, 0xef", 16, "abcdef");
	pass("#align 8", "#d16 0xabcd, 0xcdef, 0xefab", 16, "abcdcdefefab");
	pass("#align 8", "#d32 0x12345678, 0x1, 0xabcdef", 16, "123456780000000100abcdef");
	
	// Big integers currently not supported.
	//pass("#align 8", "#d64 0x12345678abcdef00, 0x123", 16, "12345678abcdef000000000000000123");
	//pass("#align 8", "#d128 0x12345678abcdef", 16, "00000000000000000012345678abcdef");
	fail("#align 8", "#d64 0x123", 1, "not supported");
	fail("#align 8", "#d128 0x123", 1, "not supported");
	
	fail("#align 3", "#d1 0b1", 1, "aligned");
	fail("#align 3", "#d2 0b10", 1, "aligned");
	fail("#align 4", "#d1 0b1", 1, "aligned");
	fail("#align 4", "#d2 0b10", 1, "aligned");
	fail("#align 4", "#d3 0b101", 1, "aligned");
	fail("#align 8", "#d4 0b1010", 1, "aligned");
	fail("#align 8", "#d8 0x79 \n #d4 0b1010", 2, "aligned");
}


#[test]
fn test_data_directive_with_expressions()
{
	pass("#align 8", "#d8 (1)", 16, "01");
	pass("#align 8", "#d8 1 + 1", 16, "02");
	pass("#align 8", "#d8 1 + 2 + 3", 16, "06");
	pass("#align 8", "#d8 (1 + 1)", 16, "02");
	
	pass("#align 8", "#d8 (1), (2)", 16, "0102");
	pass("#align 8", "#d8 1 + 1, 1 + 2", 16, "0203");
	pass("#align 8", "#d8 1 + 2 + 3, 1 + 3 + 6", 16, "060a");
	pass("#align 8", "#d8 (1 + 1), (2 + 3)", 16, "0205");
}


#[test]
fn test_data_directive_with_variables()
{
	pass("#align 8", "#d8 pc", 16, "00");
	pass("#align 8", "#d8 0x12, pc", 16, "1201");
	
	pass("#align 8", "start: \n #d8 start", 16, "00");
	pass("#align 8", "start: \n #d8 0x12, 0x34, start", 16, "123400");
	
	pass("#align 8", "#d8 start             \n start:", 16, "01");
	pass("#align 8", "#d8 0x12, 0x34, start \n start:", 16, "123403");
	
	pass("#align 8", "start: \n #d8 start,   end        \n end:", 16, "0002");
	pass("#align 8", "start: \n #d8   end, start        \n end:", 16, "0200");
	pass("#align 8", "start: \n #d8 start,  0x45,   end \n end:", 16, "004503");
	pass("#align 8", "start: \n #d8   end,  0x45, start \n end:", 16, "034500");
	
	pass("#align 8", "#address 0x1234 \n start:          \n #d8 start", 16, "34");
	pass("#align 8", "#address 0x1234 \n start:          \n #d16 start", 16, "1234");
	pass("#align 8", "#d8 start       \n #address 0x1234 \n start:", 16, "34");
	pass("#align 8", "#d16 start      \n #address 0x1234 \n start:", 16, "1234");
	
	fail("#align 8", "#d8 xyz", 1, "unknown");
	fail("#align 8", "#d8 0x12, xyz", 1, "unknown");
	fail("#align 8", "#d8 0x12 \n #d8 xyz", 2, "unknown");
}


#[test]
fn test_str_directive_simple()
{
	pass("#align 1",  "#str \"abcd\"", 16, "61626364");
	pass("#align 2",  "#str \"abcd\"", 16, "61626364");
	pass("#align 4",  "#str \"abcd\"", 16, "61626364");
	pass("#align 8",  "#str \"abcd\"", 16, "61626364");
	pass("#align 16", "#str \"abcd\"", 16, "61626364");
	pass("#align 32", "#str \"abcd\"", 16, "61626364");
	pass("#align 64", "#str \"abcd\"", 16, "6162636400000000");
	
	pass("#align 1",  "#str \"hello\"", 16, "68656c6c6f");
	pass("#align 2",  "#str \"hello\"", 16, "68656c6c6f");
	pass("#align 4",  "#str \"hello\"", 16, "68656c6c6f");
	pass("#align 8",  "#str \"hello\"", 16, "68656c6c6f");
	pass("#align 16", "#str \"hello\"", 16, "68656c6c6f00");
	pass("#align 32", "#str \"hello\"", 16, "68656c6c6f000000");
	pass("#align 64", "#str \"hello\"", 16, "68656c6c6f000000");
	
	pass("#align 3", "#str \"abcd\"", 2, "011000010110001001100011011001000");
	pass("#align 5", "#str \"abcd\"", 2, "01100001011000100110001101100100000");
	pass("#align 7", "#str \"abcd\"", 2, "01100001011000100110001101100100000");
	pass("#align 9", "#str \"abcd\"", 2, "011000010110001001100011011001000000");
}


#[test]
fn test_str_directive_utf8()
{
	pass("#align 1",  "#str \"木\"", 16, "e69ca8");
	pass("#align 2",  "#str \"木\"", 16, "e69ca8");
	pass("#align 4",  "#str \"木\"", 16, "e69ca8");
	pass("#align 8",  "#str \"木\"", 16, "e69ca8");
	pass("#align 16", "#str \"木\"", 16, "e69ca800");
	pass("#align 32", "#str \"木\"", 16, "e69ca800");
	pass("#align 64", "#str \"木\"", 16, "e69ca80000000000");
	
	pass("#align 1",  "#str \"ab木cd\"", 16, "6162e69ca86364");
	pass("#align 2",  "#str \"ab木cd\"", 16, "6162e69ca86364");
	pass("#align 4",  "#str \"ab木cd\"", 16, "6162e69ca86364");
	pass("#align 8",  "#str \"ab木cd\"", 16, "6162e69ca86364");
	pass("#align 16", "#str \"ab木cd\"", 16, "6162e69ca8636400");
	pass("#align 32", "#str \"ab木cd\"", 16, "6162e69ca8636400");
	pass("#align 64", "#str \"ab木cd\"", 16, "6162e69ca8636400");
}


#[test]
fn test_str_directive_escape()
{
	pass("#align 8", "#str \"\0\"",   16, "00");
	pass("#align 8", "#str \"\t\"",   16, "09");
	pass("#align 8", "#str \"\n\"",   16, "0a");
	pass("#align 8", "#str \"\r\"",   16, "0d");
	pass("#align 8", "#str \"\\\\\"", 16, "5c");
	pass("#align 8", "#str \"\\\"\"", 16, "22");
	
	pass("#align 8", "#str \"\\\"",   16, "5c");
	
	pass("#align 8", "#str \"\\x00\"",   16, "00");
	pass("#align 8", "#str \"\\x12\"",   16, "12");
	pass("#align 8", "#str \"\\x7f\"",   16, "7f");
	pass("#align 8", "#str \"\\x80\"",   16, "c280");
	pass("#align 8", "#str \"\\xab\"",   16, "c2ab");
	pass("#align 8", "#str \"\\xAB\"",   16, "c2ab");
	pass("#align 8", "#str \"\\xabcd\"", 16, "c2ab6364");
	pass("#align 8", "#str \"ab\\xcd\"", 16, "6162c38d");
	
	pass("#align 8", "#str \"\\x\"",  16, "5c");
	pass("#align 8", "#str \"\\x0\"", 16, "5c30");
	
	fail("#align 8", "#str 0",    1, "string");
	fail("#align 8", "#str \"0",  1, "line break");
	fail("#align 8", "#str \"\\", 1, "line break");
}


#[test]
fn test_strl_directive_simple()
{
	pass("#align 8",  "#strl 8,  \"abcd\"", 16, "0461626364");
	pass("#align 8",  "#strl 16, \"abcd\"", 16, "000461626364");
	pass("#align 8",  "#strl 24, \"abcd\"", 16, "00000461626364");
	pass("#align 8",  "#strl 32, \"abcd\"", 16, "0000000461626364");
	pass("#align 8",  "#strl 64, \"abcd\"", 16, "000000000000000461626364");
	pass("#align 16", "#strl 16, \"abcd\"", 16, "000461626364");
	pass("#align 16", "#strl 32, \"abcd\"", 16, "0000000461626364");
	pass("#align 16", "#strl 64, \"abcd\"", 16, "000000000000000461626364");
	pass("#align 32", "#strl 32, \"abcd\"", 16, "0000000461626364");
	pass("#align 32", "#strl 64, \"abcd\"", 16, "000000000000000461626364");
	pass("#align 64", "#strl 64, \"abcd\"", 16, "00000000000000046162636400000000");

	pass("#align 8",  "#strl 8,  \"\"",        16, "00");
	pass("#align 8",  "#strl 8,  \"a\"",       16, "0161");
	pass("#align 8",  "#strl 8,  \"ab\"",      16, "026162");
	pass("#align 8",  "#strl 8,  \"abc\"",     16, "03616263");
	pass("#align 8",  "#strl 8,  \"abcde\"",   16, "056162636465");
	pass("#align 8",  "#strl 8,  \"abcdef\"",  16, "06616263646566");
	pass("#align 8",  "#strl 8,  \"abcdefg\"", 16, "0761626364656667");
	
	fail("#align 8",  "#strl \"abcd\"",   1, "expected");
	fail("#align 8",  "#strl 8 \"abcd\"", 1, "expected");
	
	pass("#align 4",  "#strl 4, \"0123456789abcde\"",  16, "f303132333435363738396162636465");
	fail("#align 4",  "#strl 4, \"0123456789abcdef\"", 1,  "does not fit");
	
	fail("#align 8",  "#strl 1, \"abcd\"", 1, "string length");
	fail("#align 8",  "#strl 2, \"abcd\"", 1, "string length");
	fail("#align 8",  "#strl 3, \"abcd\"", 1, "string length");
	fail("#align 8",  "#strl 4, \"abcd\"", 1, "string length");
	fail("#align 8",  "#strl 5, \"abcd\"", 1, "string length");
	fail("#align 8",  "#strl 6, \"abcd\"", 1, "string length");
	fail("#align 8",  "#strl 7, \"abcd\"", 1, "string length");
	fail("#align 8",  "#strl 9, \"abcd\"", 1, "string length");
}


#[test]
fn test_strl_directive_utf8()
{
	pass("#align 8", "#strl 8, \"木\"", 16, "03e69ca8");
	pass("#align 8", "#strl 8, \"ab木cd\"", 16, "076162e69ca86364");
}


#[test]
fn test_reserve_directive()
{
	pass(DEF_SIMPLE, "#res 1     \n halt", 16, "0010");
	pass(DEF_SIMPLE, "#res 3     \n halt", 16, "00000010");
	pass(DEF_SIMPLE, "#res 1 + 2 \n halt", 16, "00000010");
	
	pass(DEF_SIMPLE, "a: \n #res 1 \n b: \n #res 1 \n jmp a", 16, "00001300");
	pass(DEF_SIMPLE, "a: \n #res 1 \n b: \n #res 1 \n jmp b", 16, "00001301");
}


#[test]
fn test_constants()
{
	pass(DEF_SIMPLE, "a = 0x12 \n jmp a", 16, "1312");
	pass(DEF_SIMPLE, "jmp a    \n a = 0x12", 16, "1312");
	
	pass(DEF_SIMPLE, "a = 0x34 \n b = a    \n jmp a", 16, "1334");
	pass(DEF_SIMPLE, "a = 0x34 \n b = a    \n jmp b", 16, "1334");
	pass(DEF_SIMPLE, "a = 0x34 \n jmp b    \n b = a", 16, "1334");
	pass(DEF_SIMPLE, "jmp b    \n a = 0x34 \n b = a", 16, "1334");
	
	fail(DEF_SIMPLE, "b = a", 1, "unknown");
	fail(DEF_SIMPLE, "b = a \n a = 0x56", 1, "unknown");
}


#[test]
fn test_labels_simple()
{	
	pass(DEF_SIMPLE, "start: \n jmp start", 16, "1300");
	
	pass(DEF_SIMPLE, "jmp loop \n loop: \n halt", 16, "130210");
	pass(DEF_SIMPLE, "jmp loop \n loop: \n jmp loop", 16, "13021302");
	
	pass(DEF_SIMPLE, "start: \n .x: \n jmp .x \n loop: \n .x: \n jmp .x", 16, "13001302");
	pass(DEF_SIMPLE, "          .x: \n jmp .x \n loop: \n .x: \n jmp .x", 16, "13001302");
	
	fail(DEF_SIMPLE, "start: \n jmp start \n start:", 3, "duplicate");
	fail(DEF_SIMPLE, ".xyz:  \n jmp .xyz  \n .xyz:", 3, "duplicate local");
	
	fail(DEF_SIMPLE, "        jmp  xyz", 1, "unknown");
	fail(DEF_SIMPLE, "halt \n jmp  xyz", 2, "unknown");
	fail(DEF_SIMPLE, "        jmp .xyz", 1, "unknown local");
	fail(DEF_SIMPLE, "halt \n jmp .xyz", 2, "unknown local");
	
	fail(DEF_SIMPLE, "jmp .xyz \n start: \n .xyz: \n halt",     1, "unknown local");
	fail(DEF_SIMPLE, "jmp .xyz \n start: \n .xyz: \n jmp .xyz", 1, "unknown local");
	
	fail(DEF_SIMPLE, ".xyz: \n halt     \n start: \n jmp .xyz", 4, "unknown local");
	fail(DEF_SIMPLE, ".xyz: \n jmp .xyz \n start: \n jmp .xyz", 4, "unknown local");
}


#[test]
fn test_include_directive()
{
	let mut filehandler = CustomFileHandler::new();
	filehandler.add("simple",      "halt \n add 0x45");
	filehandler.add("def_global",  "start: \n halt \n add 0x45");
	filehandler.add("use_global",  "jmp start");
	filehandler.add("sub/simple",  "halt \n add 0x67");
	filehandler.add("sub/include", "halt \n add 0x89 \n #include \"other\"");
	filehandler.add("sub/other",   "halt \n add 0xab");
	
	filehandler.add("pass1", "#include \"simple\"");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass1", 16, "101145");
	
	filehandler.add("pass2", "#include \"def_global\"");
	filehandler.add("pass3", "#include \"def_global\" \n #include \"use_global\"");
	filehandler.add("pass4", "#include \"use_global\" \n #include \"def_global\"");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass2", 16, "101145");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass3", 16, "1011451300");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass4", 16, "1302101145");
	
	filehandler.add("pass5", "#include \"sub/simple\"");
	filehandler.add("pass6", "#include \"sub/include\"");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass5", 16, "101167");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass6", 16, "1011891011ab");
	
	filehandler.add("fail1", "#include \"xyz\"");
	filehandler.add("fail2", "#include \"use_global\"");
	fail_filehandler(DEF_SIMPLE, &filehandler, "fail1", "fail1", 1, "not exist");
	fail_filehandler(DEF_SIMPLE, &filehandler, "fail2", "use_global", 1, "unknown");
}


#[test]
fn test_includebin_directive()
{
	let mut filehandler = CustomFileHandler::new();
	filehandler.add("8bits",  vec![0xd0]);
	filehandler.add("24bits", vec![0x12, 0x34, 0xe0]);
	filehandler.add("32bits", vec![0x12, 0x00, 0x56, 0xf0]);
	
	filehandler.add("pass1", "#includebin \"8bits\"");
	filehandler.add("pass2", "#includebin \"24bits\"");
	filehandler.add("pass3", "#includebin \"32bits\"");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass1", 16, "d0");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass2", 16, "1234e0");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass3", 16, "120056f0");
	
	filehandler.add("pass4", "halt \n #includebin \"8bits\"  \n halt");
	filehandler.add("pass5", "halt \n #includebin \"24bits\" \n halt");
	filehandler.add("pass6", "halt \n #includebin \"32bits\" \n halt");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass4", 16, "10d010");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass5", 16, "101234e010");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass6", 16, "10120056f010");
	
	filehandler.add("fail1", "#includebin \"xyz\"");
	fail_filehandler(DEF_SIMPLE, &filehandler, "fail1", "fail1", 1, "not exist");
	
	filehandler.add("fail2", "#includebin \"24bits\"");
	fail_filehandler("#align 16", &filehandler, "fail2", "fail2", 1, "not aligned");
}


#[test]
fn test_includebinstr_directive()
{
	let mut filehandler = CustomFileHandler::new();
	filehandler.add("empty",   "");
	filehandler.add("1bit",    "1");
	filehandler.add("2bits",   "10");
	filehandler.add("8bits",   "11101010");
	filehandler.add("16bits",  "1110101010001100");
	filehandler.add("invalid", "1110101010021100");
	
	filehandler.add("pass1", "#includebinstr \"empty\"");
	filehandler.add("pass2", "#includebinstr \"1bit\"");
	filehandler.add("pass3", "#includebinstr \"2bits\"");
	filehandler.add("pass4", "#includebinstr \"8bits\"");
	filehandler.add("pass5", "#includebinstr \"16bits\"");
	pass_filehandler("#align 1",  &filehandler, "pass1", 2, "");
	pass_filehandler("#align 2",  &filehandler, "pass1", 2, "");
	pass_filehandler("#align 8",  &filehandler, "pass1", 2, "");
	pass_filehandler("#align 16", &filehandler, "pass1", 2, "");
	pass_filehandler("#align 1",  &filehandler, "pass2", 2, "1");
	pass_filehandler("#align 1",  &filehandler, "pass3", 2, "10");
	pass_filehandler("#align 2",  &filehandler, "pass3", 2, "10");
	pass_filehandler("#align 1",  &filehandler, "pass4", 2, "11101010");
	pass_filehandler("#align 2",  &filehandler, "pass4", 2, "11101010");
	pass_filehandler("#align 4",  &filehandler, "pass4", 2, "11101010");
	pass_filehandler("#align 8",  &filehandler, "pass4", 2, "11101010");
	pass_filehandler("#align 1",  &filehandler, "pass5", 2, "1110101010001100");
	pass_filehandler("#align 8",  &filehandler, "pass5", 2, "1110101010001100");
	pass_filehandler("#align 16", &filehandler, "pass5", 2, "1110101010001100");
	
	filehandler.add("pass6", "halt \n #includebinstr \"8bits\"  \n halt");
	filehandler.add("pass7", "halt \n #includebinstr \"16bits\" \n halt");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass6", 2, "000100001110101000010000");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass7", 2, "00010000111010101000110000010000");
	
	filehandler.add("fail1", "#includebinstr \"xyz\"");
	fail_filehandler(DEF_SIMPLE, &filehandler, "fail1", "fail1", 1, "not exist");
	
	filehandler.add("fail2", "#includebinstr \"1bit\"");
	filehandler.add("fail3", "#includebinstr \"2bits\"");
	filehandler.add("fail4", "#includebinstr \"8bits\"");
	fail_filehandler("#align 2", &filehandler, "fail2", "fail2", 1, "not aligned");
	fail_filehandler("#align 3", &filehandler, "fail3", "fail3", 1, "not aligned");
	fail_filehandler("#align 3", &filehandler, "fail4", "fail4", 1, "not aligned");
	
	filehandler.add("fail5", "#includebinstr \"invalid\"");
	fail_filehandler("#align 8", &filehandler, "fail5", "fail5", 1, "invalid");
}


#[test]
fn test_includehexstr_directive()
{
	let mut filehandler = CustomFileHandler::new();
	filehandler.add("empty",   "");
	filehandler.add("4bits",   "c");
	filehandler.add("8bits",   "de");
	filehandler.add("16bits",  "ca5f");
	filehandler.add("invalid", "8z3d");
	
	filehandler.add("pass1", "#includehexstr \"empty\"");
	filehandler.add("pass2", "#includehexstr \"4bits\"");
	filehandler.add("pass3", "#includehexstr \"8bits\"");
	filehandler.add("pass4", "#includehexstr \"16bits\"");
	pass_filehandler("#align 1",  &filehandler, "pass1", 16, "");
	pass_filehandler("#align 2",  &filehandler, "pass1", 16, "");
	pass_filehandler("#align 8",  &filehandler, "pass1", 16, "");
	pass_filehandler("#align 16", &filehandler, "pass1", 16, "");
	pass_filehandler("#align 1",  &filehandler, "pass2", 16, "c");
	pass_filehandler("#align 2",  &filehandler, "pass2", 16, "c");
	pass_filehandler("#align 4",  &filehandler, "pass2", 16, "c");
	pass_filehandler("#align 4",  &filehandler, "pass3", 16, "de");
	pass_filehandler("#align 8",  &filehandler, "pass3", 16, "de");
	pass_filehandler("#align 4",  &filehandler, "pass4", 16, "ca5f");
	pass_filehandler("#align 8",  &filehandler, "pass4", 16, "ca5f");
	pass_filehandler("#align 16", &filehandler, "pass4", 16, "ca5f");
	
	filehandler.add("pass5", "halt \n #includehexstr \"8bits\"  \n halt");
	filehandler.add("pass6", "halt \n #includehexstr \"16bits\" \n halt");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass5", 16, "10de10");
	pass_filehandler(DEF_SIMPLE, &filehandler, "pass6", 16, "10ca5f10");
	
	filehandler.add("fail1", "#includehexstr \"xyz\"");
	fail_filehandler(DEF_SIMPLE, &filehandler, "fail1", "fail1", 1, "not exist");
	
	filehandler.add("fail2", "#includehexstr \"4bits\"");
	filehandler.add("fail3", "#includehexstr \"8bits\"");
	filehandler.add("fail4", "#includehexstr \"16bits\"");
	fail_filehandler("#align 8", &filehandler, "fail2", "fail2", 1, "not aligned");
	fail_filehandler("#align 3", &filehandler, "fail3", "fail3", 1, "not aligned");
	fail_filehandler("#align 3", &filehandler, "fail4", "fail4", 1, "not aligned");
	
	filehandler.add("fail5", "#includehexstr \"invalid\"");
	fail_filehandler("#align 8", &filehandler, "fail5", "fail5", 1, "invalid");
}