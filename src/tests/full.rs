#![cfg(test)]


use definition;
use assembler;
use util::bitvec::BitVec;


fn pass(def_str: &str, asm_str: &str, expected_out_radix: usize, expected_out: &str)
{
	let def = definition::parse("test", &def_str.chars().collect::<Vec<char>>()).unwrap();
	let out = assembler::assemble(&def, asm_str, &asm_str.chars().collect::<Vec<char>>()).unwrap();
	
	if !out.compare(&BitVec::new_from_str(expected_out_radix, expected_out).unwrap())
	{
		panic!(format!(
			"\ntest output mismatch:\n\n \
			def:\n{}\n\n \
			asm:\n{}\n\n \
			expected: {}\n \
			.....got: {}\n",
			def_str, asm_str,
			expected_out, out.get_hex_str()));
	}
}


fn fail(def_str: &str, asm_str: &str, expected_error_line: usize, expect_error_substr: &str)
{
	let def = definition::parse("test", &def_str.chars().collect::<Vec<char>>()).unwrap();
	match assembler::assemble(&def, "test", &asm_str.chars().collect::<Vec<char>>())
	{
		Ok(out) => panic!(format!(
			"\ntest passed but error expected:\n\n \
			def:\n{}\n\n \
			asm:\n{}\n\n \
			expected: error\n \
			.....got: {}\n",
			def_str, asm_str,
			out.get_hex_str())),
			
		Err(err) =>
			if !err.line_is(expected_error_line) || !err.contains_str(expect_error_substr)
			{
				panic!(format!(
					"\ntest error msg mismatch:\n\n \
					def:\n{}\n\n \
					asm:\n{}\n\n \
					.expected error msg: {}\n \
					......got error msg: {}\n \
					expected error line: {}\n \
					.....got error line: {}\n",
					def_str, asm_str,
					expect_error_substr, err.get_msg(),
					expected_error_line, err.get_line()));
			}
	}
}


static DEF_SIMPLE: &'static str =
"
	.align 8
	
	halt        -> 8'0x10
	add {a}     -> 8'0x11 a[7:0]
	sub {a} {b} -> 8'0x12 a[7:0] b[7:0]
	jmp {a}     -> 8'0x13 a[7:0]
";


static DEF_CONSTRAINT: &'static str =
"
	.align 8
	
	simple0 {a! : _ <= 0xff}       -> 8'0x00 a[ 7:0]
	simple0 {a! : _ <= 0xffff}     -> 8'0x01 a[15:0]
	simple0 {a! : _ <= 0xffffff}   -> 8'0x02 a[23:0]
	simple0 {a  : _ <= 0xffffffff} -> 8'0x03 a[31:0]
	
	simple1 {a! : _ <= 1 <<  8 - 1} -> 8'0x10 a[ 7:0]
	simple1 {a! : _ <= 1 << 16 - 1} -> 8'0x11 a[15:0]
	simple1 {a! : _ <= 1 << 24 - 1} -> 8'0x12 a[23:0]
	simple1 {a  : _ <= 1 << 32 - 1} -> 8'0x13 a[31:0]
	
	multi0 {a! : _ <= 1 <<  8 - 1} {b! : _ <= 1 <<  8 - 1} -> 8'0x20 a[ 7:0] b[ 7:0]
	multi0 {a! : _ <= 1 <<  8 - 1} {b! : _ <= 1 << 16 - 1} -> 8'0x21 a[ 7:0] b[15:0]
	multi0 {a! : _ <= 1 << 16 - 1} {b! : _ <= 1 <<  8 - 1} -> 8'0x22 a[15:0] b[ 7:0]
	multi0 {a! : _ <= 1 << 16 - 1} {b! : _ <= 1 << 16 - 1} -> 8'0x23 a[15:0] b[15:0]
	multi0 {a  : _ <= 1 << 32 - 1} {b  : _ <= 1 << 32 - 1} -> 8'0x24 a[31:0] b[31:0]
";


static DEF_EXPR: &'static str =
"
	.align 8
	
	slice0 {a} -> 8'0x10 a[15:0]
	slice1 {a} -> 8'0x11 a[15:8] a[ 7:0]
	slice2 {a} -> 8'0x12 a[15:8]
	slice3 {a} -> 8'0x13 a[ 7:0]
	slice4 {a} -> 8'0x14 a[ 7:0] a[15:8]
	slice5 {a} -> 8'0x15 a[0:15]
	slice6 {a} -> 8'0x16 a[31:0]
	
	expr0 {a}  -> 8'0x20 (a + 1)[7:0]
	expr1 {a}  -> 8'0x21 (a + a)[7:0]
	expr2 {a}  -> 8'0x22 (a * a)[7:0]
";


#[test]
fn test_instructions_simple()
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
}


#[test]
fn test_instructions_constraints()
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
	
	pass(DEF_CONSTRAINT, "simple1 0x1", 16, "1001");
	pass(DEF_CONSTRAINT, "simple1 0xff", 16, "10ff");
	pass(DEF_CONSTRAINT, "simple1 0x100", 16, "110100");
	pass(DEF_CONSTRAINT, "simple1 0xffff", 16, "11ffff");
	pass(DEF_CONSTRAINT, "simple1 0x10000", 16, "12010000");
	pass(DEF_CONSTRAINT, "simple1 0xffffff", 16, "12ffffff");
	pass(DEF_CONSTRAINT, "simple1 0x1000000", 16, "1301000000");
	pass(DEF_CONSTRAINT, "simple1 0xffffffff", 16, "13ffffffff");
	
	pass(DEF_CONSTRAINT, "start: \n simple1 start", 16, "1000");
	pass(DEF_CONSTRAINT, "simple1 start \n start:", 16, "1300000005");
	
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
	
	fail(DEF_CONSTRAINT, "simple0 start      \n .address 0x100110011 \n start:", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "simple1 start      \n .address 0x100110011 \n start:", 1, "not satisfied");
	fail(DEF_CONSTRAINT, "multi0 start start \n .address 0x100110011 \n start:", 1, "not satisfied");
}


#[test]
fn test_instructions_production_expr()
{
	pass(DEF_EXPR, "", 16, "");
	
	pass(DEF_EXPR, "slice0 0x1234", 16, "101234");
	pass(DEF_EXPR, "slice1 0x1234", 16, "111234");
	pass(DEF_EXPR, "slice2 0x1234", 16, "1212");
	pass(DEF_EXPR, "slice3 0x1234", 16, "1334");
	pass(DEF_EXPR, "slice4 0x1234", 16, "143412");
	pass(DEF_EXPR, "slice5 0x1234", 16, "152c48");
	pass(DEF_EXPR, "slice6 0x1234", 16, "1600001234");

	pass(DEF_EXPR, "expr0 0x08", 16, "2009");
	pass(DEF_EXPR, "expr1 0x08", 16, "2110");
	pass(DEF_EXPR, "expr2 0x08", 16, "2240");
}


#[test]
fn test_instructions_expr()
{
	pass(DEF_SIMPLE, "add 2 + 3", 16, "1105");
	pass(DEF_SIMPLE, "add 0x50 + 0x06", 16, "1156");
	pass(DEF_SIMPLE, "sub 3 - 2 12 - 7", 16, "120105");
	pass(DEF_SIMPLE, "sub (0x14 - 0x2) 0x58 - 0x2", 16, "121256");
	
	pass(DEF_CONSTRAINT, "simple1 0x100 - 0xff", 16, "1001");
	pass(DEF_CONSTRAINT, "simple1 0x100 - 1", 16, "10ff");
	pass(DEF_CONSTRAINT, "simple1 0xff + 1", 16, "110100");
	pass(DEF_CONSTRAINT, "simple1 0x10000 - 1", 16, "11ffff");
	pass(DEF_CONSTRAINT, "simple1 0xffff + 1", 16, "12010000");
	pass(DEF_CONSTRAINT, "simple1 0x1000000 - 1", 16, "12ffffff");
	pass(DEF_CONSTRAINT, "simple1 0xffffff + 1", 16, "1301000000");
	pass(DEF_CONSTRAINT, "simple1 0x100000000 - 1", 16, "13ffffffff");
	
	pass(DEF_CONSTRAINT, "simple1 start + 1 \n start:", 16, "1300000006");
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
	
	fail(".align 3", ".d1 0b1", 1, "aligned");
	fail(".align 3", ".d2 0b10", 1, "aligned");
	fail(".align 4", ".d1 0b1", 1, "aligned");
	fail(".align 4", ".d2 0b10", 1, "aligned");
	fail(".align 4", ".d3 0b101", 1, "aligned");
	fail(".align 8", ".d4 0b1010", 1, "aligned");
	fail(".align 8", ".d8 0x79 \n .d4 0b1010", 2, "aligned");
}


#[test]
fn test_literals_expr()
{
	pass(".align 8", ".d8 (1)", 16, "01");
	pass(".align 8", ".d8 1 + 1", 16, "02");
	pass(".align 8", ".d8 1 + 2 + 3", 16, "06");
	pass(".align 8", ".d8 (1 + 1)", 16, "02");
	
	pass(".align 8", ".d8 (1), (2)", 16, "0102");
	pass(".align 8", ".d8 1 + 1, 1 + 2", 16, "0203");
	pass(".align 8", ".d8 1 + 2 + 3, 1 + 3 + 6", 16, "060a");
	pass(".align 8", ".d8 (1 + 1), (2 + 3)", 16, "0205");
}


#[test]
fn test_literals_with_variables()
{
	pass(".align 8", "start: \n .d8 start", 16, "00");
	pass(".align 8", "start: \n .d8 0x12, 0x34, start", 16, "123400");
	
	pass(".align 8", ".d8 start             \n start:", 16, "01");
	pass(".align 8", ".d8 0x12, 0x34, start \n start:", 16, "123403");
	
	pass(".align 8", "start: \n .d8 start,   end        \n end:", 16, "0002");
	pass(".align 8", "start: \n .d8   end, start        \n end:", 16, "0200");
	pass(".align 8", "start: \n .d8 start,  0x45,   end \n end:", 16, "004503");
	pass(".align 8", "start: \n .d8   end,  0x45, start \n end:", 16, "034500");
	
	pass(".align 8", ".address 0x1234 \n start:          \n .d8 start", 16, "34");
	pass(".align 8", ".address 0x1234 \n start:          \n .d16 start", 16, "1234");
	pass(".align 8", ".d8 start       \n .address 0x1234 \n start:", 16, "34");
	pass(".align 8", ".d16 start      \n .address 0x1234 \n start:", 16, "1234");
	
	fail(".align 8", ".d8 xyz", 1, "unknown");
	fail(".align 8", ".d8 0x12, xyz", 1, "unknown");
	fail(".align 8", ".d8 0x12 \n .d8 xyz", 2, "unknown");
}


#[test]
fn test_labels_simple()
{	
	pass(DEF_SIMPLE, "start: \n jmp start", 16, "1300");
	
	pass(DEF_SIMPLE, "jmp loop \n loop: \n halt", 16, "130210");
	pass(DEF_SIMPLE, "jmp loop \n loop: \n jmp loop", 16, "13021302");
	
	pass(DEF_SIMPLE, "start: \n 'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", 16, "13001302");
	pass(DEF_SIMPLE, "          'x: \n jmp 'x \n loop: \n 'x: \n jmp 'x", 16, "13001302");
	
	fail(DEF_SIMPLE, "        jmp  xyz", 1, "unknown");
	fail(DEF_SIMPLE, "halt \n jmp  xyz", 2, "unknown");
	fail(DEF_SIMPLE, "        jmp 'xyz", 1, "unknown local");
	fail(DEF_SIMPLE, "halt \n jmp 'xyz", 2, "unknown local");
	
	fail(DEF_SIMPLE, "jmp 'xyz \n start: \n 'xyz: \n halt",     1, "unknown local");
	fail(DEF_SIMPLE, "jmp 'xyz \n start: \n 'xyz: \n jmp 'xyz", 1, "unknown local");
	
	fail(DEF_SIMPLE, "'xyz: \n halt     \n start: \n jmp 'xyz", 4, "unknown local");
	fail(DEF_SIMPLE, "'xyz: \n jmp 'xyz \n start: \n jmp 'xyz", 4, "unknown local");
}