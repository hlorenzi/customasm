use diagn::Report;
use util::{FileServer, FileServerMock};
use asm::BinaryOutput;
use super::ExpectedResult::*;
use super::{ExpectedResult, expect_result};
use ::read_instrset;
use ::assemble;


fn test<S, T>(instrset: S, asm: T, expected: ExpectedResult<(usize, &'static str)>)
where S: Into<Vec<u8>>, T: Into<Vec<u8>>
{
	let mut fileserver = FileServerMock::new();
	fileserver.add("instrset", instrset);
	fileserver.add("asm", asm);
	
	test_fileserver(&fileserver, "instrset", "asm", expected);
}


fn test_fileserver<S, T>(fileserver: &FileServer, instrset_filename: S, asm_filename: T, expected: ExpectedResult<(usize, &'static str)>)
where S: Into<String>, T: Into<String>
{
	let compile = |report: &mut Report, fileserver: &FileServer| -> Result<BinaryOutput, ()>
	{
		let instrset = read_instrset(report, fileserver, instrset_filename)?;
		assemble(report, &instrset, fileserver, asm_filename)
	};
	
	let mut report = Report::new();
	
	let bits = if let Pass(expected) = expected
		{ expected.0 }
	else
		{ 4 };
	
	let result = compile(&mut report, fileserver).ok();
	let result = result.map(|r| (bits, r.generate_str(bits, 0, r.len())));
	let result = result.as_ref().map(|r| (r.0, r.1.as_ref()));
	
	expect_result(&report, fileserver, result, expected);
}


#[test]
fn test_simple()
{
	test("",            "", Pass((1, "")));
	test("halt -> 8'0", "", Pass((1, "")));
	
	test("halt -> 8'0",            "halt", Pass((4, "00")));
	test("halt -> 16'0x1234",      "halt", Pass((4, "1234")));
	test("halt -> 8'0x12, 8'0x34", "halt", Pass((4, "1234")));
	test("halt -> 4'0xa,  4'0xb",  "halt", Pass((4, "ab")));
	
	test("halt -> (1 + 1)[7:0]", "halt", Pass((4, "02")));
	test("halt -> pc[7:0]",      "halt", Pass((4, "00")));
	
	test("#align 1 \n halt -> 1'0",     "halt", Pass((1, "0")));
	test("#align 1 \n halt -> 2'0b10",  "halt", Pass((1, "10")));
	test("#align 3 \n halt -> 3'0b101", "halt", Pass((1, "101")));
	test("#align 5 \n halt -> 5'0x13",  "halt", Pass((1, "10011")));
	
	test("#align 128 \n halt -> ((1 << 256) / 0xfedc)[255:0]", "halt", Pass((4, "000101254e8d998319892068f7ba90cd2a03ec79bad91fa81bbfa69a07b0c5a1")));
	
	test("#align 3 \n halt -> 3'0b101 \n cli -> 3'0b110", "halt \n cli \n halt \n cli", Pass((1, "101110101110")));
	test("#align 8 \n halt -> 8'0x12  \n cli -> 8'0x34",  "halt \n cli \n halt \n cli", Pass((4, "12341234")));
	
	test("halt -> 8'0", "unknown",         Fail(("asm", 1, "no match")));
	test("halt -> 8'0", "halt \n unknown", Fail(("asm", 2, "no match")));
	
	test("halt -> 8'0", "#unknown \n halt", Fail(("asm", 1, "unknown")));
}


#[test]
fn test_parameters()
{
	test("load {a} -> 8'0x12, a[7:0]",         "load 0x34", Pass((4, "1234")));
	test("load {a} -> 8'0x12, a[7:0]",         "load pc",   Pass((4, "1200")));
	test("load {a} -> 8'0x12, a[3:0], a[7:4]", "load 0x34", Pass((4, "1243")));
	test("load {a} -> 8'0x12, a[15:0]",        "load 0x34", Pass((4, "120034")));
	
	test("load {a}, {b} -> 8'0x12, a[7:0], b[7:0]", "load 0x34, 0x56", Pass((4, "123456")));
	test("load {a}, {b} -> 8'0x12, b[7:0], a[7:0]", "load 0x34, 0x56", Pass((4, "125634")));
	
	test("load {a}      -> 8'0x12, (a +  0x22)[7:0]", "load 0x34",       Pass((4, "1256")));
	test("load {a}      -> 8'0x12, (a + 0xf22)[7:0]", "load 0x34",       Pass((4, "1256")));
	test("load {a}, {b} -> 8'0x12, (a + b)[7:0]",     "load 0x34, 0x56", Pass((4, "128a")));
	
	test("load {a} -> 8'0x12, a[7:0]", "load 1 == 1", Fail(("asm", 1, "integer")));
	test("load {a} -> 8'0x12, a[7:0]", "load",        Fail(("asm", 1, "no match")));
	test("load {a} -> 8'0x12, a[7:0]", "load 1, 2",   Fail(("asm", 1, "no match")));
	test("load {a} -> 8'0x12, a[7:0]", "load a",      Fail(("asm", 1, "unknown")));
	
	test("load {a}, {b} -> 8'0x12, a[7:0], b[7:0]", "load 1",       Fail(("asm", 1, "no match")));
	test("load {a}, {b} -> 8'0x12, a[7:0], b[7:0]", "load 1, 2, 3", Fail(("asm", 1, "no match")));
}


#[test]
fn test_constraints()
{
	test("load {a} :: a % 2 == 0 -> 8'0x12, a[7:0]", "load 0x34", Pass((4, "1234")));
	
	test("load {a} :: a % 2 == 0               -> 8'0x12, a[7:0]", "load 0x23", Fail(("asm", 1, "constraint")));
	test("load {a} :: a % 2 == 0, \"not even\" -> 8'0x12, a[7:0]", "load 0x23", Fail(("asm", 1, "not even")));
	test("load {a} :: pc >= 0x02, \"too low\"  -> 8'0x12, a[7:0]", "load 0x34", Fail(("asm", 1, "too low")));
}


#[test]
fn test_addr_directive()
{
	test("halt -> 8'0x12", "                     halt", Pass((4, "12")));
	test("halt -> 8'0x12", "#addr 0x00        \n halt", Pass((4, "12")));
	test("halt -> 8'0x12", "#addr 0x34        \n halt", Pass((4, "12")));
	test("halt -> 8'0x12", "#addr 0xffff_ffff \n halt", Pass((4, "12")));
	
	test("halt -> 8'0x12, pc[7:0]", "                     halt", Pass((4, "1200")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x00        \n halt", Pass((4, "1200")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x34        \n halt", Pass((4, "1234")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0xffff_ffff \n halt", Pass((4, "12ff")));
	
	test("halt -> 8'0x12, pc[7:0]", "halt \n halt \n halt",                       Pass((4, "120012021204")));
	test("halt -> 8'0x12, pc[7:0]", "halt \n halt \n #addr 0x33 \n halt \n halt", Pass((4, "1200120212331235")));
	
	test("halt :: pc % 2 == 0 -> 8'0x12, pc[7:0]", "halt \n halt \n halt",               Pass((4, "120012021204")));
	test("halt :: pc % 2 == 0 -> 8'0x12, pc[7:0]", "halt \n halt \n #addr 0x33 \n halt", Fail(("asm", 4, "constraint")));
	
	test("halt -> 8'0x12", "#addr 0xffff_ffff_ffff_ffff",           Pass((4, "")));
	test("halt -> 8'0x12", "#addr 0xffff_ffff_ffff_ffff   \n halt", Fail(("asm", 2, "overflow")));
	test("halt -> 8'0x12", "#addr 0x1_0000_0000_0000_0000 \n halt", Fail(("asm", 1, "large")));
}


#[test]
fn test_outp_directive()
{
	test("halt -> 8'0x12", "              halt", Pass((4, "12")));
	test("halt -> 8'0x12", "#outp 0x00 \n halt", Pass((4, "12")));
	test("halt -> 8'0x12", "#outp 0x01 \n halt", Pass((4, "0012")));
	test("halt -> 8'0x12", "#outp 0x02 \n halt", Pass((4, "000012")));
	test("halt -> 8'0x12", "#outp 0x10 \n halt", Pass((4, "0000000000000000000000000000000012")));
	
	test("halt -> 8'0x12, pc[7:0]", "              halt", Pass((4, "1200")));
	test("halt -> 8'0x12, pc[7:0]", "#outp 0x00 \n halt", Pass((4, "1200")));
	test("halt -> 8'0x12, pc[7:0]", "#outp 0x01 \n halt", Pass((4, "001200")));
	test("halt -> 8'0x12, pc[7:0]", "#outp 0x02 \n halt", Pass((4, "00001200")));
	test("halt -> 8'0x12, pc[7:0]", "#outp 0x10 \n halt", Pass((4, "000000000000000000000000000000001200")));
	
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x45 \n #outp 0x00 \n halt", Pass((4, "1245")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x77 \n #outp 0x01 \n halt", Pass((4, "001277")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x93 \n #outp 0x02 \n halt", Pass((4, "00001293")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0xbf \n #outp 0x10 \n halt", Pass((4, "0000000000000000000000000000000012bf")));
	
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x45 \n #outp 0x00 \n halt \n halt \n halt", Pass((4, "124512471249")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x77 \n #outp 0x01 \n halt \n halt \n halt", Pass((4, "0012771279127b")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0x93 \n #outp 0x02 \n halt \n halt \n halt", Pass((4, "0000129312951297")));
	test("halt -> 8'0x12, pc[7:0]", "#addr 0xbf \n #outp 0x10 \n halt \n halt \n halt", Pass((4, "0000000000000000000000000000000012bf12c112c3")));
	
	test("halt -> 8'0x12, pc[7:0]", "#outp 0x00 \n halt \n halt \n #outp 0x10 \n halt \n halt", Pass((4, "1200120200000000000000000000000012041206")));
	
	test("halt -> 8'0x12", "#outp 0xffff_ffff_ffff_ffff",           Pass((4, "")));
	test("halt -> 8'0x12", "#outp 0xffff_ffff_ffff_ffff   \n halt", Fail(("asm", 2, "overflow")));
	test("halt -> 8'0x12", "#outp 0x1_0000_0000_0000_0000 \n halt", Fail(("asm", 1, "large")));
}


#[test]
fn test_res_directive()
{
	test("halt -> 8'0x12, pc[7:0]", "halt \n #res 0", Pass((4, "1200")));
	test("halt -> 8'0x12, pc[7:0]", "halt \n #res 1", Pass((4, "120000")));
	test("halt -> 8'0x12, pc[7:0]", "halt \n #res 2", Pass((4, "12000000")));
	test("halt -> 8'0x12, pc[7:0]", "halt \n #res 4", Pass((4, "120000000000")));
	
	test("halt -> 8'0x12, pc[7:0]", "#res 0 \n halt", Pass((4, "1200")));
	test("halt -> 8'0x12, pc[7:0]", "#res 1 \n halt", Pass((4, "001201")));
	test("halt -> 8'0x12, pc[7:0]", "#res 2 \n halt", Pass((4, "00001202")));
	test("halt -> 8'0x12, pc[7:0]", "#res 4 \n halt", Pass((4, "000000001204")));
}


#[test]
fn test_labels()
{
	static INSTRSET: &'static str = "
		halt -> 8'0x12 \n
		jump {a} -> 8'0x77, a[7:0]";
	
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
	
	test(INSTRSET, "start: halt \n .br: jump .br \n #addr 0xf0 \n mid: halt \n .br: jump .br", Pass((4, "1277011277f1")));
	
	test(INSTRSET, " label: halt \n  label: halt", Fail(("asm", 2, "duplicate")));
	test(INSTRSET, ".label: halt \n .label: halt", Fail(("asm", 2, "duplicate")));
}


#[test]
fn test_cascading()
{
	static INSTRSET: &'static str = "
		load {a} :: a < 0x10 -> 8'0x10, a[7:0] \n
		load {a} :: a < 0x20 -> 8'0x20, a[7:0] \n
		load {a}             -> 8'0xff, a[7:0] \n
		
		store {a} :: a < 0x10 -> 8'0x30, a[7:0] \n
		store {a} :: a < 0x20 -> 8'0x40, a[7:0] \n
		store {a} :: a < 0x30 -> 8'0x50, a[7:0] \n
		
		add {a}, {b} :: a < 0x10 :: b < 0x10 -> 8'0xaa, a[7:0], b[7:0] \n
		add {a}, {b} :: a < 0x20             -> 8'0xbb, a[7:0], b[7:0] \n
		add {a}, {b}             :: b < 0x20 -> 8'0xcc, a[7:0], b[7:0] \n 
		add {a}, {b}                         -> 8'0xdd, a[7:0], b[7:0]";
		
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
	test(INSTRSET, "store 0x35", Fail(("asm", 1, "constraint")));
	
	test(INSTRSET, "value = 0x05 \n store value", Pass((4, "3005")));
	test(INSTRSET, "value = 0x15 \n store value", Pass((4, "4015")));
	test(INSTRSET, "value = 0x25 \n store value", Pass((4, "5025")));
	test(INSTRSET, "value = 0x35 \n store value", Fail(("asm", 2, "constraint")));
	
	test(INSTRSET, "store value \n value = 0x05", Pass((4, "5005")));
	test(INSTRSET, "store value \n value = 0x15", Pass((4, "5015")));
	test(INSTRSET, "store value \n value = 0x25", Pass((4, "5025")));
	test(INSTRSET, "store value \n value = 0x35", Fail(("asm", 1, "constraint")));
	
	test(INSTRSET, "add 0x05, 0x07", Pass((4, "aa0507")));
	test(INSTRSET, "add 0x15, 0x25", Pass((4, "bb1525")));
	test(INSTRSET, "add 0x25, 0x15", Pass((4, "cc2515")));
	test(INSTRSET, "add 0x25, 0x25", Pass((4, "dd2525")));
	
	test(INSTRSET, "a = 0x05 \n b = 0x07 \n add a, b",                         Pass((4, "aa0507")));
	test(INSTRSET, "a = 0x05 \n             add a, b \n b = 0x07",             Pass((4, "bb0507")));
	test(INSTRSET, "a = 0x15 \n             add a, b \n b = 0x25",             Pass((4, "bb1525")));
	test(INSTRSET, "            b = 0x07 \n add a, b \n a = 0x05",             Pass((4, "cc0507")));
	test(INSTRSET, "            b = 0x15 \n add a, b \n a = 0x25",             Pass((4, "cc2515")));
	test(INSTRSET, "                        add a, b \n a = 0x07 \n b = 0x07", Pass((4, "dd0707")));
	test(INSTRSET, "                        add a, b \n a = 0x25 \n b = 0x25", Pass((4, "dd2525")));
}


#[test]
fn test_include_directive()
{
	static INSTRSET: &'static str = "
		halt     -> 8'0x12, pc[7:0]
		load {a} -> 8'0x34,  a[7:0]";
		
	static MAIN1: &'static str = "
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
		#include \"unknown\"";
			
	static MAIN3: &'static str ="
		#include \"../invalid\"";
			
	static MAIN4: &'static str ="
		#include \"./invalid\"";
		
	static MAIN5: &'static str ="
		#include \"C:\\invalid\"";
			
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
	
	test_fileserver(&fileserver, "instrset", "main1", Pass((4, "12003400120412063400120a34003406340a121234003406340a3412121c34003406340a3412")));
	test_fileserver(&fileserver, "instrset", "main2", Fail(("main2", 2, "not found")));
	test_fileserver(&fileserver, "instrset", "main3", Fail(("main3", 2, "invalid")));
	test_fileserver(&fileserver, "instrset", "main4", Fail(("main4", 2, "invalid")));
	test_fileserver(&fileserver, "instrset", "main5", Fail(("main5", 2, "invalid")));
}