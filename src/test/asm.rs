use crate::diagn::RcReport;
use crate::asm::AssemblerState;
use crate::util::{FileServer, FileServerMock};
use super::ExpectedResult::*;
use super::{ExpectedResult, expect_result};


fn test<S, T>(instrset: S, asm: T, expected: ExpectedResult<(usize, &'static str)>)
where S: Into<String>, T: Into<String>
{
	let mut cpudef = "#cpudef \"test\" { \n".to_string();
	cpudef.push_str(&instrset.into());
	cpudef.push_str("\n }");

	let mut asm_with_cpudef = "#include \"cpu\" \n".to_string();
	asm_with_cpudef.push_str(&asm.into());
	
	let mut fileserver = FileServerMock::new();
	fileserver.add("cpu", cpudef.bytes().collect::<Vec<u8>>());
	fileserver.add("asm", asm_with_cpudef);
	
	let adjusted_result =
		if let Fail((filename, line, msg)) = expected
			{ Fail((filename, line + 1, msg)) }
		else
			{ expected };
	
	test_fileserver(&fileserver, "asm", adjusted_result);
}


fn test_no_cpu<S>(asm: S, expected: ExpectedResult<(usize, &'static str)>)
where S: Into<Vec<u8>>
{
	let mut fileserver = FileServerMock::new();
	fileserver.add("asm", asm.into());
	
	test_fileserver(&fileserver, "asm", expected);
}


fn test_fileserver<S>(fileserver: &dyn FileServer, asm_filename: S, expected: ExpectedResult<(usize, &'static str)>)
where S: Into<String>
{
	let bits = if let Pass(expected) = expected
		{ expected.0 }
	else
		{ 4 };
		
	let assemble = |report: RcReport, fileserver: &dyn FileServer, filename: S| -> Result<(usize, String), ()>
	{
		let mut asm = AssemblerState::new();
		asm.process_file(report.clone(), fileserver, filename)?;
		asm.wrapup(report.clone())?;
		
		let output = asm.get_binary_output();
		Ok((bits, output.generate_str(bits, 0, output.len())))
	};
		
	let report = RcReport::new();
	
	let result = assemble(report.clone(), fileserver, asm_filename);
	
	expect_result(report.clone(), fileserver, result.as_ref().map(|r| (r.0, r.1.as_ref())).ok(), expected);
}


#[test]
fn test_simple()
{
	test("",            "", Pass((1, "")));
	test("halt -> 8'0", "", Pass((1, "")));
	
	test("halt -> 8'0",                "halt", Pass((4, "00")));
	test("halt -> 0b00000000",         "halt", Pass((4, "00")));
	test("halt -> 0o00000000",         "halt", Pass((4, "000000")));
	test("halt -> 0x00",               "halt", Pass((4, "00")));
	test("halt -> 16'0x1234",          "halt", Pass((4, "1234")));
	test("halt -> 0b0001001000110100", "halt", Pass((4, "1234")));
	test("halt -> 0x1234",             "halt", Pass((4, "1234")));
	test("halt -> 8'0x12 @ 8'0x34",    "halt", Pass((4, "1234")));
	test("halt -> 4'0xa  @ 4'0xb",     "halt", Pass((4, "ab")));
	test("halt ->   0x12 @   0x34",    "halt", Pass((4, "1234")));
	test("halt ->   0xa  @   0xb",     "halt", Pass((4, "ab")));
	
	test("halt -> (1 + 1)[7:0]", "halt", Pass((4, "02")));
	test("halt -> pc[7:0]",      "halt", Pass((4, "00")));
	
	test("#bits 1 \n halt -> 1'0",     "halt", Pass((1, "0")));
	test("#bits 1 \n halt -> 0b0",     "halt", Pass((1, "0")));
	test("#bits 1 \n halt -> 0o0",     "halt", Pass((1, "000")));
	test("#bits 1 \n halt -> 0x0",     "halt", Pass((1, "0000")));
	test("#bits 1 \n halt -> 2'0b10",  "halt", Pass((1, "10")));
	test("#bits 1 \n halt -> 0b10",    "halt", Pass((1, "10")));
	test("#bits 1 \n halt -> 0o10",    "halt", Pass((1, "001000")));
	test("#bits 1 \n halt -> 0x10",    "halt", Pass((1, "00010000")));
	test("#bits 3 \n halt -> 3'0b101", "halt", Pass((1, "101")));
	test("#bits 3 \n halt -> 0b101",   "halt", Pass((1, "101")));
	test("#bits 3 \n halt -> 0o101",   "halt", Pass((1, "001000001")));
	test("#bits 3 \n halt -> 0x101",   "halt", Pass((1, "000100000001")));
	test("#bits 5 \n halt -> 5'0x13",  "halt", Pass((1, "10011")));
	test("#bits 5 \n halt -> 0b10011", "halt", Pass((1, "10011")));
	
	test("#bits 128 \n halt -> ((1 << 256) / 0xfedc)[255:0]", "halt", Pass((4, "000101254e8d998319892068f7ba90cd2a03ec79bad91fa81bbfa69a07b0c5a1")));
	
	test("#bits 3 \n halt -> 3'0b101 \n cli -> 3'0b110", "halt \n cli \n halt \n cli", Pass((1, "101110101110")));
	test("#bits 8 \n halt -> 8'0x12  \n cli -> 8'0x34",  "halt \n cli \n halt \n cli", Pass((4, "12341234")));
	
	test("halt -> 0x00", "unknown",         Fail(("asm", 1, "no match")));
	test("halt -> 0x00", "halt \n unknown", Fail(("asm", 2, "no match")));
	
	test("halt -> 0x00", "#unknown \n halt", Fail(("asm", 1, "unknown")));
	
	test("halt -> 0x00", "HALT", Pass((4, "00")));
	test("HALT -> 0x00", "halt", Pass((4, "00")));
	test("Halt -> 0x00", "hALT", Pass((4, "00")));
	test("hALT -> 0x00", "Halt", Pass((4, "00")));

	test("halt -> 0x00", "h a l t", Pass((4, "00")));
	test("halt -> 0x00", "ha lt",   Pass((4, "00")));
	
	test("halt -> pc % 2 == 0 ? 0x12 : 0x34", "halt \n halt \n halt", Pass((4, "123412")));
	test("halt -> pc          ? 0x12 : 0x34", "halt \n halt \n halt", Fail(("cpu", 1, "type")));
	
	test("halt ->  pc % 2 == 0 ? 0x12",              "halt \n halt \n halt", Fail(("cpu", 1, "width")));
	test("halt ->  pc          ? 0x12",              "halt \n halt \n halt", Fail(("cpu", 1, "width")));
	test("halt -> (pc % 2 == 0 ? 0x12       )[7:0]", "halt \n halt \n halt", Fail(("cpu", 1, "type")));
	test("halt -> (pc          ? 0x12       )[7:0]", "halt \n halt \n halt", Fail(("cpu", 1, "type")));
}


#[test]
fn test_parameters()
{
	test("load {a} -> 0x12 @ a[7:0]",          "load 0x34", Pass((4, "1234")));
	test("load {a} -> 0x12 @ a[7:0]",          "load pc",   Pass((4, "1200")));
	test("load {a} -> 0x12 @ a[3:0] @ a[7:4]", "load 0x34", Pass((4, "1243")));
	test("load {a} -> 0x12 @ a[15:0]",         "load 0x34", Pass((4, "120034")));
	
	test("load {a}, {b} -> 0x12 @ a[7:0] @ b[7:0]", "load 0x34, 0x56", Pass((4, "123456")));
	test("load {a}, {b} -> 0x12 @ b[7:0] @ a[7:0]", "load 0x34, 0x56", Pass((4, "125634")));
	
	test("load {a}      -> 0x12 @ (a +  0x22)[7:0]", "load 0x34",       Pass((4, "1256")));
	test("load {a}      -> 0x12 @ (a + 0xf22)[7:0]", "load 0x34",       Pass((4, "1256")));
	test("load {a}, {b} -> 0x12 @ (a + b)[7:0]",     "load 0x34, 0x56", Pass((4, "128a")));
	
	test("load {a} -> 0x12 @ a[7:0]", "load 1 == 1", Fail(("cpu", 1, "type")));
	test("load {a} -> 0x12 @ a[7:0]", "load",        Fail(("asm", 1, "no match")));
	test("load {a} -> 0x12 @ a[7:0]", "load 1, 2",   Fail(("asm", 1, "no match")));
	test("load {a} -> 0x12 @ a[7:0]", "load a",      Fail(("asm", 1, "unknown")));
	
	test("load {a}, {b} -> 0x12 @ a[7:0] @ b[7:0]", "load 1",       Fail(("asm", 1, "no match")));
	test("load {a}, {b} -> 0x12 @ a[7:0] @ b[7:0]", "load 1, 2, 3", Fail(("asm", 1, "no match")));
}


#[test]
fn test_parameter_types()
{
	test("load {a: u}   -> 0x12 @ a[7:0]", "load 0x00", Fail(("cpu", 1, "type")));
	test("load {a: u1f} -> 0x12 @ a[7:0]", "load 0x00", Fail(("cpu", 1, "type")));
	test("load {a: u-1} -> 0x12 @ a[7:0]", "load 0x00", Fail(("cpu", 1, "type")));

	test("load {a: u0} -> 0x12 @ a[7:0]", "load 0x00",  Fail(("asm", 1, "out of range")));

	test("load {a: u1} -> 0x12 @ a[7:0]", "load 0b00",  Pass((4, "1200")));
	test("load {a: u1} -> 0x12 @ a[7:0]", "load 0b01",  Pass((4, "1201")));
	test("load {a: u1} -> 0x12 @ a[7:0]", "load 0b10",  Fail(("asm", 1, "out of range")));
	test("load {a: u1} -> 0x12 @ a[7:0]", "load -0b01", Fail(("asm", 1, "out of range")));

	test("load {a: u8} -> 0x12 @ a[7:0]", "load 0x00",  Pass((4, "1200")));
	test("load {a: u8} -> 0x12 @ a[7:0]", "load 0x34",  Pass((4, "1234")));
	test("load {a: u8} -> 0x12 @ a[7:0]", "load 0xff",  Pass((4, "12ff")));
	test("load {a: u8} -> 0x12 @ a[7:0]", "load 0x100", Fail(("asm", 1, "out of range")));
	test("load {a: u8} -> 0x12 @ a[7:0]", "load -0x01", Fail(("asm", 1, "out of range")));
	
	test("load {a: u16} -> 0x12 @ a[15:0]", "load 0x0000",  Pass((4, "120000")));
	test("load {a: u16} -> 0x12 @ a[15:0]", "load 0x3456",  Pass((4, "123456")));
	test("load {a: u16} -> 0x12 @ a[15:0]", "load 0xffff",  Pass((4, "12ffff")));
	test("load {a: u16} -> 0x12 @ a[15:0]", "load 0x10000", Fail(("asm", 1, "out of range")));
	test("load {a: u16} -> 0x12 @ a[15:0]", "load -0x0001", Fail(("asm", 1, "out of range")));
	
	test("load {a: s0} -> 0x12 @ a[7:0]", "load 0x00",  Fail(("asm", 1, "out of range")));

	test("load {a: s1} -> 0x12 @ a[7:0]", "load 0b00",  Pass((4, "1200")));
	test("load {a: s1} -> 0x12 @ a[7:0]", "load -0b01", Pass((4, "12ff")));
	test("load {a: s1} -> 0x12 @ a[7:0]", "load 0b01",  Fail(("asm", 1, "out of range")));
	test("load {a: s1} -> 0x12 @ a[7:0]", "load -0b10", Fail(("asm", 1, "out of range")));

	test("load {a: s8} -> 0x12 @ a[7:0]", "load 0x00",  Pass((4, "1200")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load 0x34",  Pass((4, "1234")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load 0x7e",  Pass((4, "127e")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load 0x7f",  Pass((4, "127f")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load -0x01", Pass((4, "12ff")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load -0x7f", Pass((4, "1281")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load -0x80", Pass((4, "1280")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load 0x80",  Fail(("asm", 1, "out of range")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load -0x81", Fail(("asm", 1, "out of range")));
	
	test("load {a: s8} -> 0x12 @ a[15:0]", "load -0x01", Pass((4, "12ffff")));
	
	test("load {a: i0} -> 0x12 @ a[7:0]", "load 0x00",  Fail(("asm", 1, "out of range")));

	test("load {a: i1} -> 0x12 @ a[7:0]", "load 0b00",  Pass((4, "1200")));
	test("load {a: i1} -> 0x12 @ a[7:0]", "load 0b01",  Pass((4, "1201")));
	test("load {a: i1} -> 0x12 @ a[7:0]", "load -0b01", Pass((4, "12ff")));
	test("load {a: i1} -> 0x12 @ a[7:0]", "load 0b10",  Fail(("asm", 1, "out of range")));
	test("load {a: i1} -> 0x12 @ a[7:0]", "load -0b10", Fail(("asm", 1, "out of range")));

	test("load {a: i8} -> 0x12 @ a[7:0]", "load 0x00",  Pass((4, "1200")));
	test("load {a: i8} -> 0x12 @ a[7:0]", "load 0x34",  Pass((4, "1234")));
	test("load {a: i8} -> 0x12 @ a[7:0]", "load 0xff",  Pass((4, "12ff")));
	test("load {a: i8} -> 0x12 @ a[7:0]", "load -0x01", Pass((4, "12ff")));
	test("load {a: i8} -> 0x12 @ a[7:0]", "load -0x7f", Pass((4, "1281")));
	test("load {a: i8} -> 0x12 @ a[7:0]", "load -0x80", Pass((4, "1280")));
	test("load {a: i8} -> 0x12 @ a[7:0]", "load 0x100", Fail(("asm", 1, "out of range")));
	test("load {a: i8} -> 0x12 @ a[7:0]", "load -0x81", Fail(("asm", 1, "out of range")));
	
	test("load {a: s8} -> 0x12 @ a[7:0]", "load  0x05 - 0x05", Pass((4, "1200")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load  0x30 + 0x04", Pass((4, "1234")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load  0x70 + 0x0f", Pass((4, "127f")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load  0x00 - 0x01", Pass((4, "12ff")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load -0x70 - 0x0f", Pass((4, "1281")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load -0x70 - 0x10", Pass((4, "1280")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load  0x70 + 0x10", Fail(("asm", 1, "out of range")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load -0x70 - 0x11", Fail(("asm", 1, "out of range")));

	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = 0x00",  Pass((4, "1200")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = 0x34",  Pass((4, "1234")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = 0x7f",  Pass((4, "127f")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = -0x01", Pass((4, "12ff")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = -0x7f", Pass((4, "1281")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = -0x80", Pass((4, "1280")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = 0x80",  Fail(("asm", 1, "out of range")));
	test("load {a: s8} -> 0x12 @ a[7:0]", "load x \n x = -0x81", Fail(("asm", 1, "out of range")));
}


#[test]
fn test_parameter_soft_slices()
{
	test("load {a: u3} -> 0b11111 @ a", "load 0x5", Pass((4, "fd")));

	test("load {a: u8 } -> 0x12 @ a", "load 0x34",   Pass((4, "1234")));
	test("load {a: u16} -> 0x12 @ a", "load 0x3456", Pass((4, "123456")));
	
	test("load {a: s8 } -> 0x12 @ a", "load 0x34",    Pass((4, "1234")));
	test("load {a: s8 } -> 0x12 @ a", "load -0x01",   Pass((4, "12ff")));
	test("load {a: s8 } -> 0x12 @ a", "load -0x80",   Pass((4, "1280")));
	test("load {a: s16} -> 0x12 @ a", "load -0x8000", Pass((4, "128000")));
	
	test("load {a: s8} -> 0x12 @ a[15:0]", "load -1", Pass((4, "12ffff")));

	test("load {a: s8} -> 0x12 @ (a + a)[15:0]", "load -1", Pass((4, "12fffe")));
}


#[test]
fn test_tokendef()
{
	test("#tokendef reg { r1 = 1    } \n mov {a: reg} -> 0xff @ a[7:0]", "mov r1", Pass((4, "ff01")));
	test("#tokendef reg { r1 = 0xbc } \n mov {a: reg} -> 0xff @ a[7:0]", "mov r1", Pass((4, "ffbc")));
	
	test("#tokendef reg { r1 = 1, r2 = 2 } \n mov {a: reg} -> 0xff @ a[7:0]", "mov r1 \n mov r2", Pass((4, "ff01ff02")));
	
	test("#tokendef reg1 { r1 = 1 } \n #tokendef reg2 { r1 = 2 } \n mov1 {a: reg1} -> 0xff @ a[7:0] \n mov2 {a: reg2} -> 0xee @ a[7:0]", "mov1 r1 \n mov2 r1", Pass((4, "ff01ee02")));
	
	test("#tokendef reg1 { r1 = 1 } \n #tokendef reg2 { r2 = 2 } \n mov {a: reg1} -> 0xff @ a[7:0] \n mov {a: reg2} -> 0xee @ a[7:0]", "mov r1 \n mov r2", Pass((4, "ff01ee02")));
	test("#tokendef reg1 { r1 = 1 } \n #tokendef reg2 { r1 = 2 } \n mov {a: reg1} -> 0xff @ a[7:0] \n mov {a: reg2} -> 0xee @ a[7:0]", "mov r1 \n mov r1", Pass((4, "ff01ff01")));
	test("#tokendef reg1 { r1 = 1 } \n                           \n mov {a: reg1} -> 0xff @ a[7:0] \n mov r1        -> 0xee @ 0x01  ", "mov r1 \n mov r1", Pass((4, "ff01ff01")));
	test("#tokendef reg1 { r1 = 1 } \n                           \n mov r1        -> 0xff @ 0x01   \n mov {a: reg1} -> 0xee @ a[7:0]", "mov r1 \n mov r1", Pass((4, "ff01ff01")));
	
	test("#tokendef reg { r1 = 1 } \n mov [{a: reg} + {offset}] -> 0xff @ a[7:0] @ offset[7:0]", "mov [r1 + 8]", Pass((4, "ff0108")));
	
	test("#tokendef reg { r1 = 1, r2 = 2 } \n mov {a: reg}, {b: reg} -> 0xff @ a[7:0] @ b[7:0] \n mov {a: reg}, {value} -> 0xee @ a[7:0] @ value[7:0]", "mov r1, r1 \n mov r1, 0x55",            Pass((4, "ff0101ee0155")));
	test("#tokendef reg { r1 = 1, r2 = 2 } \n mov {a: reg}, {b: reg} -> 0xff @ a[7:0] @ b[7:0] \n mov {a: reg}, {value} -> 0xee @ a[7:0] @ value[7:0]", "mov r1, r1 \n mov r1, label \n label:", Pass((4, "ff0101ee0106")));
	test("#tokendef reg { r1 = 1, r2 = 2 } \n mov {a: reg}, {b: reg} -> 0xff @ a[7:0] @ b[7:0] \n mov {a: reg}, {value} -> 0xee @ a[7:0] @ value[7:0]", "mov r1, r1 \n mov r2, label \n label:", Pass((4, "ff0101ee0206")));
	test("#tokendef reg { r1 = 1, r2 = 2 } \n mov {a: reg}, {b: reg} -> 0xff @ a[7:0] @ b[7:0] \n mov {a: reg}, {value} -> 0xee @ a[7:0] @ value[7:0]", "mov r1, r1 \n mov r2, r1    \n r1:",    Pass((4, "ff0101ff0201")));
	
	test("#tokendef reg { r1 = 0xbc } \n mov {a: reg} -> 0xff @ a[7:0]", "mov r2", Fail(("asm", 1, "no match")));
	
	test("#tokendef reg { r1 = 1, r1 = 2 } \n mov {a: reg} -> 0xff @ a[7:0]", "mov r1", Fail(("cpu", 1, "duplicate tokendef entry")));
	test("#tokendef 123 { r1 = 1, r2 = 2 } \n mov {a: reg} -> 0xff @ a[7:0]", "mov r1", Fail(("cpu", 1, "identifier")));
	test("#tokendef reg { r1 = 1 } \n #tokendef reg { r2 = 1 } \n mov {a: reg} -> 0xff @ a[7:0]", "mov r1", Fail(("cpu", 2, "duplicate tokendef name")));

	test("#tokendef reg { r1 = 1 } \n mov {dest: reg}, {src: reg} -> 0x55 @ dest[3:0] @ src[3:0] \n mov {dest: reg}, r2 -> 0x88 @ dest[7:0]", "mov r1, r1", Pass((4, "5511")));
	test("#tokendef reg { r1 = 1 } \n mov {dest: reg}, {src: reg} -> 0x55 @ dest[3:0] @ src[3:0] \n mov {dest: reg}, r2 -> 0x88 @ dest[7:0]", "mov r1, r2", Pass((4, "8801")));
	
	test("#tokendef reg { r1 = 1         } \n mov {dest: reg}, {src: reg} -> 0x55 @ dest[3:0] @ src[3:0] \n mov r1, r2 -> 0x88", "mov r1, r2",  Pass((4, "88")));
	test("#tokendef reg { r1 = 1, r2 = 2 } \n mov {dest: reg}, {src: reg} -> 0x55 @ dest[3:0] @ src[3:0] \n mov r1, r2 -> 0x88", "mov r1, r2",  Pass((4, "5512")));
	test("#tokendef reg { r1 = 1         } \n mov r1, r2 -> 0x88 \n mov {dest: reg}, {src: reg} -> 0x55 @ dest[3:0] @ src[3:0]", "mov r1, r2",  Pass((4, "88")));
	test("#tokendef reg { r1 = 1, r2 = 2 } \n mov r1, r2 -> 0x88 \n mov {dest: reg}, {src: reg} -> 0x55 @ dest[3:0] @ src[3:0]", "mov r1, r2",  Pass((4, "88")));
	
	test("#tokendef reg { r1 = 1 } \n mov r1, {src: reg} -> 0x88 @ src[7:0] \n mov r1, r1 -> 0x55", "mov r1, r2", Fail(("asm", 1, "no match")));
	test("#tokendef reg { r1 = 1 } \n mov r1, r1 -> 0x55 \n mov r1, {src: reg} -> 0x88 @ src[7:0]", "mov r1, r2", Fail(("asm", 1, "no match")));
	
	test("#tokendef alu { add = 0x10, sub = 0x20 } \n {op: alu} {x} -> op[7:0] @ x[7:0]", "add 0xef \n sub 0xfe", Pass((4, "10ef20fe")));
	
	test("#tokendef cond { z = 0x10, ne = 0x20 } \n j{c: cond} {x} -> c[7:0] @ x[7:0]", "jz 0x50  \n jne 0x60",   Pass((4, "10502060")));
	test("#tokendef cond { z = 0x10, ne = 0x20 } \n j{c: cond} {x} -> c[7:0] @ x[7:0]", "j z 0x50 \n j ne 0x60",  Pass((4, "10502060")));
	test("#tokendef cond { z = 0x10, ne = 0x20 } \n j{c: cond} {x} -> c[7:0] @ x[7:0]", "j z 0x50 \n j n e 0x60", Pass((4, "10502060")));
	test("#tokendef cond { z = 0x10 } \n j{c: cond} {x} -> c[7:0] @ x[7:0]", "j ztest", Fail(("asm", 1, "no match")));
	
	test("#tokendef cond { test = 0x10 } \n j{c: cond} -> c[7:0]", "j t e s t",   Pass((4, "10")));
	test("#tokendef cond { test = 0x10 } \n j{c: cond} -> c[7:0]", "jtestx",      Fail(("asm", 1, "no match")));
	test("#tokendef cond { test = 0x10 } \n j{c: cond} -> c[7:0]", "jtes",        Fail(("asm", 1, "no match")));
	test("#tokendef cond { test = 0x10 } \n j{c: cond} -> c[7:0]", "jtes\nt",     Fail(("asm", 1, "no match")));
	test("#tokendef cond { test = 0x10 } \n j{c: cond} -> c[7:0]", "jtes\njtest", Fail(("asm", 1, "no match")));
	
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r1",  Pass((4, "5501")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r1  \n ld r3  \n ld r18",   Pass((4, "550155035512")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r 1 \n ld r 3 \n ld r 18",  Pass((4, "550155035512")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r 1 \n ld r 3 \n ld r 1 8", Fail(("asm", 3, "no match")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r 0x1", Pass((4, "5501")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r 1+1", Pass((4, "5502")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r0x1",  Fail(("asm", 1, "no match")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld r1+1",  Fail(("asm", 1, "no match")));
	test("ld r{reg} -> 0x55 @ reg[7:0]", "ld ra",    Fail(("asm", 1, "no match")));
	
	test("ld r{reg}, {val} -> 0x55 @ reg[7:0] @ val[7:0]", "ld r1, 5",  Pass((4, "550105")));
	test("ld r{reg}, {val} -> 0x55 @ reg[7:0] @ val[7:0]", "ld r 1, 5", Pass((4, "550105")));
	test("ld r{reg}, {val} -> 0x55 @ reg[7:0] @ val[7:0]", "ld r0x1, 5", Fail(("asm", 1, "no match")));
}


#[test]
fn test_assertions()
{
	test("load {a} -> { assert(a % 2 == 0), 0x12 @ a[7:0] }", "load 0x34",               Pass((4, "1234")));
	test("load {a} -> { assert(a % 2 == 0), 0x12 @ a[7:0] }", "load 0x23",               Fail(("cpu", 1, "assertion")));
	test("load {a} -> { assert(pc >= 0x02), 0x12 @ a[7:0] }", "load 0x34",               Fail(("cpu", 1, "assertion")));
	test("load {a} -> { assert(pc >= 0x02), 0x12 @ a[7:0] }", "#addr 0x02 \n load 0x34", Pass((4, "00001234")));
	
	test("halt -> { assert(pc % 2 == 0), 0x12 @ pc[7:0] }", "halt \n halt \n halt",               Pass((4, "120012021204")));
	test("halt -> { assert(pc % 2 == 0), 0x12 @ pc[7:0] }", "halt \n halt \n #addr 0x33 \n halt", Fail(("cpu", 1, "assertion")));
}


#[test]
fn test_addr_directive()
{
	test("halt -> 0x12", "              halt", Pass((4, "12")));
	test("halt -> 0x12", "#addr 0x00 \n halt", Pass((4, "12")));
	test("halt -> 0x12", "#addr 0x01 \n halt", Pass((4, "0012")));
	test("halt -> 0x12", "#addr 0x10 \n halt", Pass((4, "0000000000000000000000000000000012")));
	
	test("halt -> 0x12", "#addr 0x10 \n halt \n #addr 0x00", Fail(("asm", 3, "previous")));
	test("halt -> 0x12", "#addr 0x10 \n halt \n #addr 0x10", Fail(("asm", 3, "previous")));
	test("halt -> 0x12", "#addr 0x10 \n halt \n #addr 0x11", Pass((4, "0000000000000000000000000000000012")));
	test("halt -> 0x12", "#addr 0x10 \n halt \n #addr 0x12", Pass((4, "000000000000000000000000000000001200")));
	
	test("halt -> 0x12 @ pc[7:0]", "              halt", Pass((4, "1200")));
	test("halt -> 0x12 @ pc[7:0]", "#addr 0x00 \n halt", Pass((4, "1200")));
	test("halt -> 0x12 @ pc[7:0]", "#addr 0x01 \n halt", Pass((4, "001201")));
	test("halt -> 0x12 @ pc[7:0]", "#addr 0x10 \n halt", Pass((4, "000000000000000000000000000000001210")));
	
	test("halt -> 0x12 @ pc[7:0]", "halt \n halt \n halt",                       Pass((4, "120012021204")));
	test("halt -> 0x12 @ pc[7:0]", "halt \n halt \n #addr 0x10 \n halt \n halt", Pass((4, "1200120200000000000000000000000012101212")));
	
	//test("halt -> 0x12", "#addr 0xffff_fffe \n halt", Pass((4, "??")));
	//test("halt -> 0x12", "#addr 0xffff_ffff",         Pass((4, "")));
	//test("halt -> 0x12", "#addr 0x1_0000_0001",           Fail(("asm", 1, "bank range")));
	test("halt -> 0x12", "#addr 0x1_0000_0000_0000_0000", Fail(("asm", 1, "large")));
}


#[test]
fn test_labelalign()
{
	test("#labelalign 1 \n halt -> 0x12", "label: halt", Pass((4, "12")));
	test("#labelalign 2 \n halt -> 0x12", "label: halt", Pass((4, "12")));
	test("#labelalign 3 \n halt -> 0x12", "label: halt", Pass((4, "12")));
	test("#labelalign 4 \n halt -> 0x12", "label: halt", Pass((4, "12")));
	
	test("#labelalign 1 \n halt -> 0x12", "halt \n label: halt", Pass((4, "1212")));
	test("#labelalign 2 \n halt -> 0x12", "halt \n label: halt", Pass((4, "120012")));
	test("#labelalign 3 \n halt -> 0x12", "halt \n label: halt", Pass((4, "12000012")));
	test("#labelalign 4 \n halt -> 0x12", "halt \n label: halt", Pass((4, "1200000012")));
	
	test("#labelalign 1 \n halt -> 0x12", "#d1 0, 0, 0 \n label: halt", Pass((4, "0012")));
	test("#labelalign 2 \n halt -> 0x12", "#d1 0, 0, 0 \n label: halt", Pass((4, "000012")));
	test("#labelalign 3 \n halt -> 0x12", "#d1 0, 0, 0 \n label: halt", Pass((4, "00000012")));
	test("#labelalign 4 \n halt -> 0x12", "#d1 0, 0, 0 \n label: halt", Pass((4, "0000000012")));
	
	test("#labelalign 1 \n halt -> 0x12", "#d8 0, 0, 0 \n halt \n label: halt", Pass((4, "0000001212")));
	test("#labelalign 2 \n halt -> 0x12", "#d8 0, 0, 0 \n halt \n label: halt", Pass((4, "0000001212")));
	test("#labelalign 3 \n halt -> 0x12", "#d8 0, 0, 0 \n halt \n label: halt", Pass((4, "00000012000012")));
	test("#labelalign 4 \n halt -> 0x12", "#d8 0, 0, 0 \n halt \n label: halt", Pass((4, "0000001212")));
}


#[test]
fn test_align_directive()
{
	test("halt -> 0x12", "#align 1 \n halt", Pass((4, "12")));
	test("halt -> 0x12", "#align 2 \n halt", Pass((4, "12")));
	test("halt -> 0x12", "#align 3 \n halt", Pass((4, "12")));
	test("halt -> 0x12", "#align 4 \n halt", Pass((4, "12")));
	
	test("halt -> 0x12", "halt \n #align 1 \n halt", Pass((4, "1212")));
	test("halt -> 0x12", "halt \n #align 2 \n halt", Pass((4, "120012")));
	test("halt -> 0x12", "halt \n #align 3 \n halt", Pass((4, "12000012")));
	test("halt -> 0x12", "halt \n #align 4 \n halt", Pass((4, "1200000012")));
	
	test("halt -> 0x12", "#d8 0, 0, 0 \n halt \n #align 1 \n halt", Pass((4, "0000001212")));
	test("halt -> 0x12", "#d8 0, 0, 0 \n halt \n #align 2 \n halt", Pass((4, "0000001212")));
	test("halt -> 0x12", "#d8 0, 0, 0 \n halt \n #align 3 \n halt", Pass((4, "00000012000012")));
	test("halt -> 0x12", "#d8 0, 0, 0 \n halt \n #align 4 \n halt", Pass((4, "0000001212")));
	
	test("halt -> 0x12", "#d1 0, 0, 0 \n #align 1 \n halt", Pass((4, "0012")));
	test("halt -> 0x12", "#d1 0, 0, 0 \n #align 2 \n halt", Pass((4, "000012")));
	test("halt -> 0x12", "#d1 0, 0, 0 \n #align 3 \n halt", Pass((4, "00000012")));
	test("halt -> 0x12", "#d1 0, 0, 0 \n #align 4 \n halt", Pass((4, "0000000012")));
	
	test("halt -> 0x12", "halt \n #align 0                       \n halt", Fail(("asm", 2, "invalid")));
	test("halt -> 0x12", "halt \n #align 0x1_0000_0000_0000_0000 \n halt", Fail(("asm", 2, "large")));
}


#[test]
fn test_res_directive()
{
	test("halt -> 0x12 @ pc[7:0]", "halt \n #res 0", Pass((4, "1200")));
	test("halt -> 0x12 @ pc[7:0]", "halt \n #res 1", Pass((4, "120000")));
	test("halt -> 0x12 @ pc[7:0]", "halt \n #res 2", Pass((4, "12000000")));
	test("halt -> 0x12 @ pc[7:0]", "halt \n #res 4", Pass((4, "120000000000")));
	
	test("halt -> 0x12 @ pc[7:0]", "#res 0 \n halt", Pass((4, "1200")));
	test("halt -> 0x12 @ pc[7:0]", "#res 1 \n halt", Pass((4, "001201")));
	test("halt -> 0x12 @ pc[7:0]", "#res 2 \n halt", Pass((4, "00001202")));
	test("halt -> 0x12 @ pc[7:0]", "#res 4 \n halt", Pass((4, "000000001204")));
}


#[test]
fn test_data_directive()
{
	test("", "#d8 0",     Pass((4, "00")));
	test("", "#d8 0xff",  Pass((4, "ff")));
	test("", "#d8 -1",    Pass((4, "ff")));
	test("", "#d8 0x80",  Pass((4, "80")));
	test("", "#d8 -0x7f", Pass((4, "81")));
	test("", "#d8 -0x80", Pass((4, "80")));
	test("", "#d8 1 + 1", Pass((4, "02")));
	test("", "#d8 pc",    Pass((4, "00")));
	
	test("", "#d8 0x1ff[7:0]", Pass((4, "ff")));
	test("", "#d8 -0x81[7:0]", Pass((4, "7f")));
	
	test("", "#d16 0",       Pass((4, "0000")));
	test("", "#d16 0xffff",  Pass((4, "ffff")));
	test("", "#d16 -1",      Pass((4, "ffff")));
	test("", "#d16 0x8000",  Pass((4, "8000")));
	test("", "#d16 -0x7fff", Pass((4, "8001")));
	test("", "#d16 -0x8000", Pass((4, "8000")));
	test("", "#d16 1 + 1",   Pass((4, "0002")));
	test("", "#d16 pc",      Pass((4, "0000")));
	
	test("", "#d16 0x1ffff[15:0]", Pass((4, "ffff")));
	test("", "#d16 -0x8001[15:0]", Pass((4, "7fff")));
	
	test("#bits 3", "#d3 0",      Pass((1, "000")));
	test("#bits 3", "#d3 0b111",  Pass((1, "111")));
	test("#bits 3", "#d3 -1",     Pass((1, "111")));
	test("#bits 3", "#d3 0b100",  Pass((1, "100")));
	test("#bits 3", "#d3 -0b011", Pass((1, "101")));
	test("#bits 3", "#d3 -0b100", Pass((1, "100")));
	test("#bits 3", "#d3 1 + 1",  Pass((1, "010")));
	test("#bits 3", "#d3 pc",     Pass((1, "000")));
	
	test("", "#d8 1,    2,    3", Pass((4, "010203")));
	test("", "#d8 1, \n 2, \n 3", Pass((4, "010203")));
	
	test("", "#d16  1,  2,  3", Pass((4, "000100020003")));
	test("", "#d16 -1, -2, -3", Pass((4, "fffffffefffd")));
	test("", "#d32  1,  2,  3", Pass((4, "000000010000000200000003")));
	test("", "#d32 -1, -2, -3", Pass((4, "fffffffffffffffefffffffd")));
	
	test("#bits 1", "#d1 1, 0, 1, -1", Pass((1, "1011")));
	test("#bits 2", "#d2 1, 2, 3, -1", Pass((1, "01101111")));
	test("#bits 3", "#d3 1, 2, 3, -1", Pass((1, "001010011111")));
	test("#bits 5", "#d5 1, 2, 3, -1", Pass((1, "00001000100001111111")));
	test("#bits 7", "#d7 1, 2, 3, -1", Pass((1, "0000001000001000000111111111")));
	test("#bits 9", "#d9 1, 2, 3, -1", Pass((1, "000000001000000010000000011111111111")));
	
	test("", "#d8  x \n  x =  0x12", Pass((4, "12")));
	test("", "#d8  x \n  x = 0x100", Fail(("asm", 1, "large")));
	test("", "#d8  x \n  x = -0x81", Fail(("asm", 1, "large")));
	test("", "#d8  x",               Fail(("asm", 1, "unknown")));
	test("", "#d8 .x \n .x =  0x12", Pass((4, "12")));
	test("", "#d8 .x \n .x = 0x100", Fail(("asm", 1, "large")));
	test("", "#d8 .x \n .x = -0x81", Fail(("asm", 1, "large")));
	test("", "#d8 .x",               Fail(("asm", 1, "unknown")));
	
	test("", "              #d8 x + pc \n #addr 0x10 \n x = 0x11", Pass((4, "11000000000000000000000000000000")));
	test("", "#addr 0x10 \n #d8 x + pc \n               x = 0x11", Pass((4, "0000000000000000000000000000000021")));
	
	test("", "#d8 0x100", Fail(("asm", 1, "large")));
	test("", "#d8 -0x81", Fail(("asm", 1, "large")));
	
	test("", "#d16 0x10000", Fail(("asm", 1, "large")));
	test("", "#d16 -0x8001", Fail(("asm", 1, "large")));
	
	test("#bits 1", "#d1 2",  Fail(("asm", 1, "large")));
	test("#bits 1", "#d1 -2", Fail(("asm", 1, "large")));
	
	test("", "#d1 0", Pass((1, "0")));
	test("", "#d2 0", Pass((1, "00")));
	test("", "#d3 0", Pass((1, "000")));
	test("", "#d4 0", Pass((1, "0000")));
	test("", "#d5 0", Pass((1, "00000")));
	test("", "#d6 0", Pass((1, "000000")));
	test("", "#d7 0", Pass((1, "0000000")));
	test("", "#d9 0", Pass((1, "000000000")));
	
	test("", "#d1 0 \n label:", Fail(("asm", 2, "align")));
	test("", "#d2 0 \n label:", Fail(("asm", 2, "align")));
	test("", "#d3 0 \n label:", Fail(("asm", 2, "align")));
	test("", "#d4 0 \n label:", Fail(("asm", 2, "align")));
	test("", "#d5 0 \n label:", Fail(("asm", 2, "align")));
	test("", "#d6 0 \n label:", Fail(("asm", 2, "align")));
	test("", "#d7 0 \n label:", Fail(("asm", 2, "align")));
	test("", "#d9 0 \n label:", Fail(("asm", 2, "align")));
	
	test("", "#d0    0",      Fail(("asm", 1, "invalid")));
	test("", "#d16y  0xffff", Fail(("asm", 1, "unknown")));
	test("", "#da16  0xffff", Fail(("asm", 1, "unknown")));
	test("", "#d0x10 0xffff", Fail(("asm", 1, "unknown")));
	
	test_no_cpu("#d8 0",           Pass((4, "00")));
	test_no_cpu("#d8 0, 1, 2, 3",  Pass((4, "00010203")));
	test_no_cpu("#d8 pc",          Fail(("asm", 1, "cpu")));
}


#[test]
fn test_str_directive()
{
	test("", "#str \"\"",                   Pass((4, "")));
	test("", "#str \"\\thello\\r\\n\\0\"",  Pass((4, "0968656c6c6f0d0a00")));
	test("", "#str \"\\x00\\x01\\x02\"",    Pass((4, "000102")));
	test("", "#str \"\\u{0}\\u{1}\\u{2}\"", Pass((4, "000102")));
	
	test("#bits 16", "#str \"\"",                         Pass((4, "")));
	test("#bits 16", "#str \"hello\\r\\n\\0\"",           Pass((4, "68656c6c6f0d0a00")));
	test("#bits 32", "#str \"\\x00\\x01\\x02\\x03\"",     Pass((4, "00010203")));
	test("#bits 32", "#str \"\\u{0}\\u{1}\\u{2}\\u{3}\"", Pass((4, "00010203")));
	
	test("", "#str \"\\u{7f}\\u{80}\\u{ff}\\u{10ffff}\"", Pass((4, "7fc280c3bff48fbfbf")));
	
	test("", "#str \"木水火\"", Pass((4, "e69ca8e6b0b4e781ab")));
	
	test("", "#str \"\\z\"", Fail(("asm", 1, "invalid")));
	
	test("#bits 5",  "#str \"abc\" \n #d1 0    \n #str \"defgh\" \n label:", Pass((1, "01100001011000100110001100110010001100101011001100110011101101000")));
	test("#bits 32", "#str \"ab\"  \n #d8 0xff \n #str \"d\"     \n label:", Pass((4, "6162ff64")));
	
	test("#bits 5",  "#str \"abc\" \n label:", Fail(("asm", 2, "align")));
	test("#bits 32", "#str \"abc\" \n label:", Fail(("asm", 2, "align")));
}


#[test]
fn test_labels()
{
	static INSTRSET: &'static str = "
		halt -> 8'0x12 \n
		jump {a} -> 8'0x77 @ a[7:0]";
	
	test(INSTRSET, "label: halt \n jump label",                Pass((4, "127700")));
	test(INSTRSET, "       halt \n jump label \n label: halt", Pass((4, "12770312")));
	test(INSTRSET, "       halt \n jump label",                Fail(("asm", 2, "unknown")));
	
	test(INSTRSET, "label = 0x55 \n halt \n jump label",                 Pass((4, "127755")));
	test(INSTRSET, "                halt \n jump label \n label = 0x55", Pass((4, "127755")));
	
	test(INSTRSET, ".label: halt \n jump .label",                 Pass((4, "127700")));
	test(INSTRSET, "        halt \n jump .label \n .label: halt", Pass((4, "12770312")));
	test(INSTRSET, "        halt \n jump .label",                 Fail(("asm", 2, "unknown")));
	test(INSTRSET, " label: halt \n jump .label",                 Fail(("asm", 2, "unknown")));
	
	test(INSTRSET, ".label = 0x55 \n halt \n jump .label",                  Pass((4, "127755")));
	test(INSTRSET, "                 halt \n jump .label \n .label = 0x55", Pass((4, "127755")));
	test(INSTRSET, " label = 0x55 \n halt \n jump .label",                  Fail(("asm", 3, "unknown")));
	
	test(INSTRSET, "label1 = 0x55          \n label2 = label1 + 0x11 \n jump label2", Pass((4, "7766")));
	test(INSTRSET, "label2 = label1 + 0x11 \n label1 = 0x55          \n jump label2", Fail(("asm", 1, "unknown")));
	
	test(INSTRSET, "start: halt \n .br: jump .br \n mid: halt \n .br: jump .br", Pass((4, "127701127704")));
	test(INSTRSET, "start: halt \n      jump .br \n mid: halt \n .br: jump .br", Fail(("asm", 2, "unknown")));
	test(INSTRSET, "start: halt \n .br: jump .br \n mid: halt \n      jump .br", Fail(("asm", 4, "unknown")));
	
	test(INSTRSET, "jump = 0x33 \n jump jump", Pass((4, "7733")));
	
	test(INSTRSET, "start: halt \n .br: jump .br \n #addr 0x08 \n mid: halt \n .br: jump .br", Pass((4, "1277010000000000127709")));
	
	test(INSTRSET, " label: halt \n  label: halt", Fail(("asm", 2, "duplicate")));
	test(INSTRSET, ".label: halt \n .label: halt", Fail(("asm", 2, "duplicate")));
	
	test(INSTRSET, "label: halt \n jump LABEL", Fail(("asm", 2, "unknown")));
	test(INSTRSET, "LABEL: halt \n jump label", Fail(("asm", 2, "unknown")));
	test(INSTRSET, "myVar: halt \n jump myvar", Fail(("asm", 2, "unknown")));
	test(INSTRSET, "myvar: halt \n jump myVar", Fail(("asm", 2, "unknown")));
}


#[test]
fn test_cascading()
{
	static INSTRSET: &'static str = "
		load {a} -> { assert(a < 0x10), 0x10 @ a[7:0] } \n
		load {a} -> { assert(a < 0x20), 0x20 @ a[7:0] } \n
		load {a} -> {                   0xff @ a[7:0] } \n
		
		store {a} -> { assert(a < 0x10), 0x30 @ a[7:0] } \n
		store {a} -> { assert(a < 0x20), 0x40 @ a[7:0] } \n
		store {a} -> { assert(a < 0x30), 0x50 @ a[7:0] } \n
		
		add {a}, {b} -> { assert(a < 0x10), assert(b < 0x10), 0xaa @ a[7:0] @ b[7:0] } \n
		add {a}, {b} -> { assert(a < 0x20),                   0xbb @ a[7:0] @ b[7:0] } \n
		add {a}, {b} -> {                   assert(b < 0x20), 0xcc @ a[7:0] @ b[7:0] } \n 
		add {a}, {b} -> {                                     0xdd @ a[7:0] @ b[7:0] }";
		
	test(INSTRSET, "load 0x05", Pass((4, "1005")));
	test(INSTRSET, "load 0x15", Pass((4, "2015")));
	test(INSTRSET, "load 0x25", Pass((4, "ff25")));
	
	test(INSTRSET, "value = 0x05 \n load value", Pass((4, "1005")));
	test(INSTRSET, "value = 0x15 \n load value", Pass((4, "2015")));
	test(INSTRSET, "value = 0x25 \n load value", Pass((4, "ff25")));
	
	test(INSTRSET, "load value \n value = 0x05", Pass((4, "ff05")));
	test(INSTRSET, "load value \n value = 0x15", Pass((4, "ff15")));
	test(INSTRSET, "load value \n value = 0x25", Pass((4, "ff25")));
	
	test(INSTRSET, "store 0x05", Pass((4, "3005")));
	test(INSTRSET, "store 0x15", Pass((4, "4015")));
	test(INSTRSET, "store 0x25", Pass((4, "5025")));
	test(INSTRSET, "store 0x35", Fail(("cpu", 13, "assertion")));
	
	test(INSTRSET, "value = 0x05 \n store value", Pass((4, "3005")));
	test(INSTRSET, "value = 0x15 \n store value", Pass((4, "4015")));
	test(INSTRSET, "value = 0x25 \n store value", Pass((4, "5025")));
	test(INSTRSET, "value = 0x35 \n store value", Fail(("cpu", 13, "assertion")));
	
	test(INSTRSET, "store value \n value = 0x05", Pass((4, "5005")));
	test(INSTRSET, "store value \n value = 0x15", Pass((4, "5015")));
	test(INSTRSET, "store value \n value = 0x25", Pass((4, "5025")));
	test(INSTRSET, "store value \n value = 0x35", Fail(("cpu", 13, "assertion")));
	
	test(INSTRSET, "add 0x05, 0x07", Pass((4, "aa0507")));
	test(INSTRSET, "add 0x15, 0x25", Pass((4, "bb1525")));
	test(INSTRSET, "add 0x25, 0x15", Pass((4, "cc2515")));
	test(INSTRSET, "add 0x25, 0x25", Pass((4, "dd2525")));
	
	test(INSTRSET, "a = 0x05 \n b = 0x07 \n add a, b",                         Pass((4, "aa0507")));
	test(INSTRSET, "a = 0x05 \n b = 0x25 \n add a, b",                         Pass((4, "bb0525")));
	test(INSTRSET, "a = 0x15 \n b = 0x07 \n add a, b",                         Pass((4, "bb1507")));
	test(INSTRSET, "a = 0x15 \n b = 0x25 \n add a, b",                         Pass((4, "bb1525")));
	test(INSTRSET, "a = 0x25 \n b = 0x15 \n add a, b",                         Pass((4, "cc2515")));
	test(INSTRSET, "a = 0x25 \n b = 0x25 \n add a, b",                         Pass((4, "dd2525")));
	
	test(INSTRSET, "a = 0x05 \n             add a, b \n b = 0x07",             Pass((4, "dd0507")));
	test(INSTRSET, "a = 0x15 \n             add a, b \n b = 0x25",             Pass((4, "dd1525")));
	test(INSTRSET, "            b = 0x07 \n add a, b \n a = 0x05",             Pass((4, "dd0507")));
	test(INSTRSET, "            b = 0x15 \n add a, b \n a = 0x25",             Pass((4, "dd2515")));
	test(INSTRSET, "                        add a, b \n a = 0x07 \n b = 0x07", Pass((4, "dd0707")));
	test(INSTRSET, "                        add a, b \n a = 0x25 \n b = 0x25", Pass((4, "dd2525")));
	
	static INSTRSET2: &'static str = "
		load {a: u4} -> 0x10 @ a[7:0] \n
		load {a: u8} -> 0x20 @ a[7:0] \n
		load {a}     -> 0xff @ a[7:0] \n
		
		store {a: u4 } -> 0x30 @ a[7:0] \n
		store {a: u8 } -> 0x40 @ a[7:0] \n
		store {a: u16} -> 0x50 @ a[7:0] \n
		
		add {a: u4}, {b: u4} -> 0xaa @ a[7:0] @ b[7:0] \n
		add {a: u8}, {b}     -> 0xbb @ a[7:0] @ b[7:0] \n
		add {a},     {b: u8} -> 0xcc @ a[7:0] @ b[7:0] \n 
		add {a},     {b}     -> 0xdd @ a[7:0] @ b[7:0]";

	test(INSTRSET2, "load 0x005", Pass((4, "1005")));
	test(INSTRSET2, "load 0x015", Pass((4, "2015")));
	test(INSTRSET2, "load 0x125", Pass((4, "ff25")));

	test(INSTRSET2, "value = 0x005 \n load value", Pass((4, "1005")));
	test(INSTRSET2, "value = 0x015 \n load value", Pass((4, "2015")));
	test(INSTRSET2, "value = 0x125 \n load value", Pass((4, "ff25")));
	
	test(INSTRSET2, "load value \n value = 0x005", Pass((4, "ff05")));
	test(INSTRSET2, "load value \n value = 0x015", Pass((4, "ff15")));
	test(INSTRSET2, "load value \n value = 0x125", Pass((4, "ff25")));
	
	test(INSTRSET2, "store 0x00005",  Pass((4, "3005")));
	test(INSTRSET2, "store 0x00015",  Pass((4, "4015")));
	test(INSTRSET2, "store 0x00125",  Pass((4, "5025")));
	test(INSTRSET2, "store 0x11135", Fail(("asm", 1, "out of range")));
	
	test(INSTRSET2, "value = 0x00005 \n store value", Pass((4, "3005")));
	test(INSTRSET2, "value = 0x00015 \n store value", Pass((4, "4015")));
	test(INSTRSET2, "value = 0x00125 \n store value", Pass((4, "5025")));
	test(INSTRSET2, "value = 0x11135 \n store value", Fail(("asm", 2, "out of range")));
	
	test(INSTRSET2, "store value \n value = 0x00005", Pass((4, "5005")));
	test(INSTRSET2, "store value \n value = 0x00015", Pass((4, "5015")));
	test(INSTRSET2, "store value \n value = 0x00125", Pass((4, "5025")));
	test(INSTRSET2, "store value \n value = 0x11135", Fail(("asm", 1, "out of range")));
	
	test(INSTRSET2, "add 0x005, 0x007", Pass((4, "aa0507")));
	test(INSTRSET2, "add 0x015, 0x025", Pass((4, "bb1525")));
	test(INSTRSET2, "add 0x125, 0x015", Pass((4, "cc2515")));
	test(INSTRSET2, "add 0x125, 0x125", Pass((4, "dd2525")));
	
	test(INSTRSET2, "a = 0x005 \n b = 0x007 \n add a, b", Pass((4, "aa0507")));
	test(INSTRSET2, "a = 0x005 \n b = 0x125 \n add a, b", Pass((4, "bb0525")));
	test(INSTRSET2, "a = 0x015 \n b = 0x007 \n add a, b", Pass((4, "bb1507")));
	test(INSTRSET2, "a = 0x015 \n b = 0x125 \n add a, b", Pass((4, "bb1525")));
	test(INSTRSET2, "a = 0x125 \n b = 0x015 \n add a, b", Pass((4, "cc2515")));
	test(INSTRSET2, "a = 0x125 \n b = 0x125 \n add a, b", Pass((4, "dd2525")));
	
	test(INSTRSET2, "a = 0x005 \n             add a, b \n b = 0x007",              Pass((4, "dd0507")));
	test(INSTRSET2, "a = 0x015 \n             add a, b \n b = 0x125",              Pass((4, "dd1525")));
	test(INSTRSET2, "            b = 0x007 \n add a, b \n a = 0x005",              Pass((4, "dd0507")));
	test(INSTRSET2, "            b = 0x015 \n add a, b \n a = 0x125",              Pass((4, "dd2515")));
	test(INSTRSET2, "                         add a, b \n a = 0x007 \n b = 0x007", Pass((4, "dd0707")));
	test(INSTRSET2, "                         add a, b \n a = 0x125 \n b = 0x125", Pass((4, "dd2525")));
}


#[test]
fn test_include_directive()
{
	static INSTRSET: &'static str = "
		#cpudef
		{
			halt     -> 8'0x12 @ pc[7:0]
			load {a} -> 8'0x34 @  a[7:0]
		}";
		
	static MAIN1: &'static str = "
		#include \"instrset\"
		
		start:
			halt
			load start
			halt
			
		#include \"folder1/file1\"
		#include \"file1\"
		
			halt
			load start
			load at_folder1_file1
			load at_folder1_file2
			load at_file1";
			
	static FOLDER1_FILE1: &'static str ="
		at_folder1_file1:
			halt
			load start
			
		#include \"file2\"";
	
	static FOLDER1_FILE2: &'static str ="
		at_folder1_file2:
			halt
			load start
			load at_folder1_file1
			load at_folder1_file2";
			
	static FILE1: &'static str ="
		at_file1:
			halt
			load start
			load at_folder1_file1
			load at_folder1_file2
			load at_file1";
			
	static MAIN2: &'static str ="
		#include \"instrset\"
		#include \"unknown\"";
			
	static MAIN3: &'static str ="
		#include \"instrset\"
		#include \"../invalid\"";
			
	static MAIN4: &'static str ="
		#include \"instrset\"
		#include \"./invalid\"";
		
	static MAIN5: &'static str ="
		#include \"instrset\"
		#include \"C:\\\\invalid\"";
	
	let mut fileserver = FileServerMock::new();
	fileserver.add("instrset", INSTRSET);
	fileserver.add("main1", MAIN1);
	fileserver.add("folder1/file1", FOLDER1_FILE1);
	fileserver.add("folder1/file2", FOLDER1_FILE2);
	fileserver.add("file1", FILE1);
	fileserver.add("main2", MAIN2);
	fileserver.add("main3", MAIN3);
	fileserver.add("main4", MAIN4);
	fileserver.add("main5", MAIN5);
	
	test_fileserver(&fileserver, "main1", Pass((4, "12003400120412063400120a34003406340a121234003406340a3412121c34003406340a3412")));
	test_fileserver(&fileserver, "main2", Fail(("main2", 3, "not found")));
	test_fileserver(&fileserver, "main3", Fail(("main3", 3, "invalid")));
	test_fileserver(&fileserver, "main4", Fail(("main4", 3, "invalid")));
	test_fileserver(&fileserver, "main5", Fail(("main5", 3, "invalid")));
}


#[test]
fn test_incbin_directive()
{
	static INSTRSET1: &'static str = "#cpudef { }";

	static INSTRSET2: &'static str = "#cpudef { \n #bits 5 \n }";
	
	static INSTRSET3: &'static str = "#cpudef { \n #bits 32 \n }";
	
	static MAIN1_1: &'static str = "#include \"instrset1\" \n #incbin \"binary1\" \n label:";
	static MAIN1_2: &'static str = "#include \"instrset1\" \n #incbin \"binary2\" \n label:";
	static MAIN1_3: &'static str = "#include \"instrset1\" \n #incbin \"binary3\" \n label:";
	static MAIN2_1: &'static str = "#include \"instrset2\" \n #incbin \"binary1\" \n label:";
	static MAIN2_2: &'static str = "#include \"instrset2\" \n #incbin \"binary2\" \n label:";
	static MAIN2_3: &'static str = "#include \"instrset2\" \n #incbin \"binary3\" \n label:";
	static MAIN3_1: &'static str = "#include \"instrset3\" \n #incbin \"binary1\" \n label:";
	static MAIN3_2: &'static str = "#include \"instrset3\" \n #incbin \"binary2\" \n label:";
	static MAIN3_3: &'static str = "#include \"instrset3\" \n #incbin \"binary3\" \n label:";
		
	static MAIN4: &'static str = "#include \"instrset1\" \n #incbin \"unknown\"";
	static MAIN5: &'static str = "#include \"instrset1\" \n #incbin \"../invalid\"";
	
	static BINARY1: &'static str = "\x12\x34\x56\x78";
	
	static BINARY2: &'static str = "testing!!!";
	
	static BINARY3: &'static str = "\u{80}\u{ff}\u{5927}";
	
	let mut fileserver = FileServerMock::new();
	fileserver.add("instrset1", INSTRSET1);
	fileserver.add("instrset2", INSTRSET2);
	fileserver.add("instrset3", INSTRSET3);
	fileserver.add("main1_1", MAIN1_1);
	fileserver.add("main1_2", MAIN1_2);
	fileserver.add("main1_3", MAIN1_3);
	fileserver.add("main2_1", MAIN2_1);
	fileserver.add("main2_2", MAIN2_2);
	fileserver.add("main2_3", MAIN2_3);
	fileserver.add("main3_1", MAIN3_1);
	fileserver.add("main3_2", MAIN3_2);
	fileserver.add("main3_3", MAIN3_3);
	fileserver.add("main4", MAIN4);
	fileserver.add("main5", MAIN5);
	fileserver.add("binary1", BINARY1);
	fileserver.add("binary2", BINARY2);
	fileserver.add("binary3", BINARY3);
	
	test_fileserver(&fileserver, "main1_1", Pass((4, "12345678")));
	test_fileserver(&fileserver, "main1_2", Pass((4, "74657374696e67212121")));
	test_fileserver(&fileserver, "main1_3", Pass((4, "c280c3bfe5a4a7")));
	
	test_fileserver(&fileserver, "main2_1", Fail(("main2_1", 3, "align")));
	test_fileserver(&fileserver, "main2_2", Pass((4, "74657374696e67212121")));
	test_fileserver(&fileserver, "main2_3", Fail(("main2_3", 3, "align")));
	
	test_fileserver(&fileserver, "main3_1", Pass((4, "12345678")));
	test_fileserver(&fileserver, "main3_2", Fail(("main3_2", 3, "align")));
	test_fileserver(&fileserver, "main3_3", Fail(("main3_3", 3, "align")));
	
	test_fileserver(&fileserver, "main4", Fail(("main4", 2, "not found")));
	test_fileserver(&fileserver, "main5", Fail(("main5", 2, "invalid")));
}


#[test]
fn test_incstr_directives()
{
	static INSTRSET1: &'static str = "#cpudef { }";

	static INSTRSET2: &'static str = "#cpudef { \n #bits 5 \n }";
	
	static INSTRSET3: &'static str = "#cpudef { \n #bits 32 \n }";
	
	static MAIN1_1: &'static str = "#include \"instrset1\" \n #incbinstr \"str1\" \n label:";
	static MAIN1_2: &'static str = "#include \"instrset1\" \n #inchexstr \"str1\" \n label:";
	static MAIN1_3: &'static str = "#include \"instrset1\" \n #incbinstr \"str2\" \n label:";
	static MAIN1_4: &'static str = "#include \"instrset1\" \n #inchexstr \"str2\" \n label:";
	static MAIN1_5: &'static str = "#include \"instrset1\" \n #incbinstr \"str3\" \n label:";
	static MAIN1_6: &'static str = "#include \"instrset1\" \n #inchexstr \"str3\" \n label:";
	static MAIN1_7: &'static str = "#include \"instrset1\" \n #incbinstr \"str4\" \n label:";
	
	static MAIN2_1: &'static str = "#include \"instrset2\" \n #incbinstr \"str1\" \n label:";
	static MAIN2_2: &'static str = "#include \"instrset2\" \n #inchexstr \"str1\" \n label:";
	static MAIN2_3: &'static str = "#include \"instrset2\" \n #incbinstr \"str2\" \n label:";
	static MAIN2_4: &'static str = "#include \"instrset2\" \n #inchexstr \"str2\" \n label:";
	static MAIN2_5: &'static str = "#include \"instrset2\" \n #incbinstr \"str3\" \n label:";
	static MAIN2_6: &'static str = "#include \"instrset2\" \n #inchexstr \"str3\" \n label:";
	static MAIN2_7: &'static str = "#include \"instrset2\" \n #incbinstr \"str4\" \n label:";
	
	static MAIN3_1: &'static str = "#include \"instrset3\" \n #incbinstr \"str1\" \n label:";
	static MAIN3_2: &'static str = "#include \"instrset3\" \n #inchexstr \"str1\" \n label:";
	static MAIN3_3: &'static str = "#include \"instrset3\" \n #incbinstr \"str2\" \n label:";
	static MAIN3_4: &'static str = "#include \"instrset3\" \n #inchexstr \"str2\" \n label:";
	static MAIN3_5: &'static str = "#include \"instrset3\" \n #incbinstr \"str3\" \n label:";
	static MAIN3_6: &'static str = "#include \"instrset3\" \n #inchexstr \"str3\" \n label:";
	static MAIN3_7: &'static str = "#include \"instrset3\" \n #incbinstr \"str4\" \n label:";
	
	static MAIN4: &'static str = "#include \"instrset1\" \n #incbin \"unknown\"";
	static MAIN5: &'static str = "#include \"instrset1\" \n #incbin \"../invalid\"";
	
	static STR1: &'static str = "1110101101000111";
	
	static STR2: &'static str = "0123456789abcdef";
	
	static STR3: &'static str = "0123456789abcdeg";
	
	static STR4: &'static str = "111010110100011";
	
	let mut fileserver = FileServerMock::new();
	fileserver.add("instrset1", INSTRSET1);
	fileserver.add("instrset2", INSTRSET2);
	fileserver.add("instrset3", INSTRSET3);
	fileserver.add("main1_1", MAIN1_1);
	fileserver.add("main1_2", MAIN1_2);
	fileserver.add("main1_3", MAIN1_3);
	fileserver.add("main1_4", MAIN1_4);
	fileserver.add("main1_5", MAIN1_5);
	fileserver.add("main1_6", MAIN1_6);
	fileserver.add("main1_7", MAIN1_7);
	fileserver.add("main2_1", MAIN2_1);
	fileserver.add("main2_2", MAIN2_2);
	fileserver.add("main2_3", MAIN2_3);
	fileserver.add("main2_4", MAIN2_4);
	fileserver.add("main2_5", MAIN2_5);
	fileserver.add("main2_6", MAIN2_6);
	fileserver.add("main2_7", MAIN2_7);
	fileserver.add("main3_1", MAIN3_1);
	fileserver.add("main3_2", MAIN3_2);
	fileserver.add("main3_3", MAIN3_3);
	fileserver.add("main3_4", MAIN3_4);
	fileserver.add("main3_5", MAIN3_5);
	fileserver.add("main3_6", MAIN3_6);
	fileserver.add("main3_7", MAIN3_7);
	fileserver.add("main4", MAIN4);
	fileserver.add("main5", MAIN5);
	fileserver.add("str1", STR1);
	fileserver.add("str2", STR2);
	fileserver.add("str3", STR3);
	fileserver.add("str4", STR4);
	
	test_fileserver(&fileserver, "main1_1", Pass((1, "1110101101000111")));
	test_fileserver(&fileserver, "main1_2", Pass((4, "1110101101000111")));
	test_fileserver(&fileserver, "main1_3", Fail(("main1_3", 2, "invalid character")));
	test_fileserver(&fileserver, "main1_4", Pass((4, "0123456789abcdef")));
	test_fileserver(&fileserver, "main1_5", Fail(("main1_5", 2, "invalid character")));
	test_fileserver(&fileserver, "main1_6", Fail(("main1_6", 2, "invalid character")));
	test_fileserver(&fileserver, "main1_7", Fail(("main1_7", 3, "align")));
	
	test_fileserver(&fileserver, "main2_1", Fail(("main2_1", 3, "align")));
	test_fileserver(&fileserver, "main2_2", Fail(("main2_2", 3, "align")));
	test_fileserver(&fileserver, "main2_3", Fail(("main2_3", 2, "invalid character")));
	test_fileserver(&fileserver, "main2_4", Fail(("main2_4", 3, "align")));
	test_fileserver(&fileserver, "main2_5", Fail(("main2_5", 2, "invalid character")));
	test_fileserver(&fileserver, "main2_6", Fail(("main2_6", 2, "invalid character")));
	test_fileserver(&fileserver, "main2_7", Pass((1, "111010110100011")));
	
	test_fileserver(&fileserver, "main3_1", Fail(("main3_1", 3, "align")));
	test_fileserver(&fileserver, "main3_2", Pass((4, "1110101101000111")));
	test_fileserver(&fileserver, "main3_3", Fail(("main3_3", 2, "invalid character")));
	test_fileserver(&fileserver, "main3_4", Pass((4, "0123456789abcdef")));
	test_fileserver(&fileserver, "main3_5", Fail(("main3_5", 2, "invalid character")));
	test_fileserver(&fileserver, "main3_6", Fail(("main3_6", 2, "invalid character")));
	test_fileserver(&fileserver, "main3_7", Fail(("main3_7", 3, "align")));
	
	test_fileserver(&fileserver, "main4", Fail(("main4", 2, "not found")));
	test_fileserver(&fileserver, "main5", Fail(("main5", 2, "invalid")));
}


#[test]
fn test_banks()
{
	test("", "#bankdef \"\"      { }",                                  Fail(("asm", 1, "invalid")));
	test("", "#bankdef \"hello\" { }",                                  Fail(("asm", 1, "missing")));
	test("", "#bankdef \"hello\" { #addr 0 }",                          Fail(("asm", 1, "missing")));
	test("", "#bankdef \"hello\" { #addr 0, #size 0 }",                 Pass((4, "")));
	test("", "#bankdef \"hello\" { #addr 0, #size 0, #outp 0 }",        Pass((4, "")));
	test("", "#bankdef \"hello\" { #addr 0, #size 0,          #fill }", Pass((4, "")));
	test("", "#bankdef \"hello\" { #addr 0, #size 0, #outp 0, #fill }", Pass((4, "")));
	
	test("", "#bankdef \"hello\"    {    #addr 0     #size 0     #outp 0    }", Fail(("asm", 1, ",")));
	test("", "#bankdef \"hello\"    {    #addr 0  \n #size 0  \n #outp 0    }", Pass((4, "")));
	test("", "#bankdef \"hello\" \n { \n #addr 0  \n #size 0  \n #outp 0 \n }", Pass((4, "")));
	test("", "#bankdef \"hello\" \n { \n #addr 0, \n #size 0, \n #outp 0 \n }", Pass((4, "")));
	
	test("", "#d8 0xff \n #bankdef \"hello\" { #addr 0, #size 0, #outp 0 }", Fail(("asm", 2, "default bank")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 10, #outp 0        }", Pass((4, "")));
	test("", "#bankdef \"hello\" { #addr 0, #size 10, #outp 0, #fill }", Pass((4, "00000000000000000000")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 10 } \n #res 1", Pass((4, "")));
	test("", "#bankdef \"hello\" { #addr 0, #size 10 } \n #d8 0",  Fail(("asm", 2, "non-writable")));
	
	test("", "#bankdef \"hello\" { #addr  0, #size 10 } \n #res 1 \n #addr 0 \n #res 1", Pass((4, "")));
	test("", "#bankdef \"hello\" { #addr 10, #size 10 } \n #res 1 \n #addr 0 \n #res 1", Fail(("asm", 3, "out of bank range")));
	test("test -> 0x12", "#bankdef \"hello\" { #addr 0, #size 3, #outp 0 } \n #res 1 \n test \n test",  Pass((4, "001212")));
	test("test -> 0x12", "#bankdef \"hello\" { #addr 0, #size 2, #outp 0 } \n #res 1 \n test \n test",  Fail(("asm", 4, "out of bank range")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #bankdef \"world\" { #addr 0, #size 4, #outp 0 }",
	          Fail(("asm", 1, "overlap")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #bankdef \"world\" { #addr 0, #size 4, #outp 3 }",
	          Fail(("asm", 1, "overlap")));
			  
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 3 }
	          #bankdef \"world\" { #addr 0, #size 4, #outp 0 }",
	          Fail(("asm", 1, "overlap")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #bankdef \"world\" { #addr 0, #size 4, #outp 4 }",
	          Pass((4, "")));
			  
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 4 }
	          #bankdef \"world\" { #addr 0, #size 4, #outp 0 }",
	          Pass((4, "")));
			  
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #bank \"hello\"
	          #d8 pc, 1, 2, 3",
	          Pass((4, "00010203")));
			  
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #d8 pc, 1, 2, 3",
	          Pass((4, "00010203")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #d8 pc, 1, 2, 3
	          label:",
	          Fail(("asm", 3, "bank range")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #addr 0
	          #d8 0xff
	          #addr 2
	          #d8 0xff
	          #addr 4",
	          Pass((4, "ff00ff00")));
	
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 0 }
	          #d8 pc, 1, 2, 3, 4",
	          Fail(("asm", 2, "overflow")));
			  
	test("", "#bankdef \"hello\" { #addr 0, #size 4, #outp 4 }
	          #d8 pc, 1, 2, 3",
	          Pass((4, "0000000000010203")));
			  
	test("", "#bankdef \"hello\" { #addr 7, #size 4, #outp 0 }
	          #d8 pc, 1, 2, 3",
	          Pass((4, "07010203")));
	
	test("", "#bankdef \"hello\" { #addr 7, #size 4, #outp 4 }
	          #d8 pc, 1, 2, 3",
	          Pass((4, "0000000007010203")));
			  
	test("", "#bankdef \"hello\" { #addr 7, #size 4, #outp 4 }
	          #d8 pc, x
	          x:
	          #d8 x, pc",
	          Pass((4, "000000000709090a")));
			  
	test("", "#bankdef \"hello\" { #addr 7, #size 4, #outp 4 }
	          #d8 pc, x
	          x = pc
	          #d8 x, pc",
	          Pass((4, "000000000709090a")));
			  
	test("", "#bank \"hello\"", Fail(("asm", 1, "unknown")));
	
	test("", "#bankdef \"hello\" { #addr 0x00, #size 0x08, #outp 0x00 }
	          #bankdef \"world\" { #addr 0x10, #size 0x04, #outp 0x0a }
	          #bank \"hello\"
			  x = pc
			  #d8 0xaa, 0xbb
			  #bank \"world\"
			  y = pc
			  #d8 0xcc, 0xdd
			  #bank \"hello\"
			  z = pc
			  #d8 0xee, 0xff
			  #d8 x, y, z",
	          Pass((4, "aabbeeff001002000000ccdd")));
	
	test("", "#bankdef \"hello\" { #addr 0x00, #size 0x08, #outp 0x00 }
	          #bankdef \"world\" { #addr 0x10, #size 0x04, #outp 0x0a }
	          #bank \"hello\"
			  x:
			  #d8 0xaa, 0xbb
			  #bank \"world\"
			  y:
			  #d8 0xcc, 0xdd
			  #bank \"hello\"
			  z:
			  #d8 0xee, 0xff
			  #d8 x, y, z",
	          Pass((4, "aabbeeff001002000000ccdd")));
	
}