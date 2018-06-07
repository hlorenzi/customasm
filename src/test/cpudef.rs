use diagn::RcReport;
use asm::cpudef::CpuDef;
use syntax::{Parser, tokenize};
use util::{FileServer, FileServerMock};
use super::ExpectedResult::*;
use super::{ExpectedResult, expect_result};


fn test<S>(src: S, expected: ExpectedResult<()>)
where S: Into<Vec<u8>>
{
	let compile = |report: RcReport, fileserver: &FileServer| -> Result<(), ()>
	{
		let chars = fileserver.get_chars(report.clone(), "test", None)?;
		let tokens = tokenize(report.clone(), "test", &chars)?;
		let mut parser = Parser::new(report.clone(), tokens);
		
		CpuDef::parse(&mut parser)?;
		Ok(())
	};

	let report = RcReport::new();
	
	let mut fileserver = FileServerMock::new();
	fileserver.add("test", src.into());
	
	let result = compile(report.clone(), &fileserver).ok();
	expect_result(report.clone(), &fileserver, result, expected);
}


#[test]
fn test_directives()
{
	test("", Pass(()));
	
	test("#bits 1",    Pass(()));
	test("#bits 3",    Pass(()));
	test("#bits 8",    Pass(()));
	test("#bits 16",   Pass(()));
	test("#bits 32",   Pass(()));
	test("#bits 64",   Pass(()));
	test("#bits 128",  Pass(()));
	test("#bits 1024", Pass(()));
	
	test("#labelalign 1", Pass(()));
	
	test("#bits 0xffff_ffff_ffff_ffff", Pass(()));
	
	test("#unknown", Fail(("test", 1, "unknown")));
	
	test("#bits 0",                       Fail(("test", 1, "invalid")));
	test("#bits 8\n #bits 8",             Fail(("test", 2, "duplicate")));
	test("#bits 0x1_0000_0000_0000_0000", Fail(("test", 1, "large")));
	
	test("#labelalign 0",                 Fail(("test", 1, "invalid")));
	test("#labelalign 4\n #labelalign 8", Fail(("test", 2, "duplicate")));
}


#[test]
fn test_rules_simple()
{
	test("halt    -> 8'0",        Pass(()));
	test("halt    -> 0[7:0]",     Pass(()));
	test("halt \n -> 0[7:0]",     Pass(()));
	test("halt    -> 0x00",       Pass(()));
	test("halt    -> 0b00000000", Pass(()));
	test("halt    -> 0o00000000", Pass(()));
	
	test("#bits 1 \n halt -> 1'0",     Pass(()));
	test("#bits 1 \n halt -> 0b0",     Pass(()));
	test("#bits 1 \n halt -> 0o0",     Pass(()));
	test("#bits 1 \n halt -> 0x0",     Pass(()));
	test("#bits 1 \n halt -> 2'0b10",  Pass(()));
	test("#bits 1 \n halt -> 0b10",    Pass(()));
	test("#bits 1 \n halt -> 0o10",    Pass(()));
	test("#bits 1 \n halt -> 0x10",    Pass(()));
	test("#bits 3 \n halt -> 3'0b101", Pass(()));
	test("#bits 3 \n halt -> 0b101",   Pass(()));
	test("#bits 3 \n halt -> 0o101",   Pass(()));
	test("#bits 3 \n halt -> 0x101",   Pass(()));
	test("#bits 5 \n halt -> 5'0x13",  Pass(()));
	test("#bits 5 \n halt -> 0b10011", Pass(()));
	test("#bits 5 \n halt -> 0o13",    Fail(("test", 2, "width")));
	test("#bits 5 \n halt -> 0x13",    Fail(("test", 2, "width")));
	
	
	test("halt    -> 0x_0_0",        Pass(()));
	test("halt    -> 0b_0000_0000",  Pass(()));
	test("halt    -> 0o_000_00_000", Pass(()));
	
	test("halt -> 8'0x12 @ 8'0x34", Pass(()));
	test("halt -> 16'0x1234",       Pass(()));
	test("halt -> 3'0x7 @ 5'0x1f",  Pass(()));
	
	test("halt + - > < * / # -> 8'0xab", Pass(()));
	
	test("halt -> pc[23:0]",      Pass(()));
	test("halt -> (1 + 1)[23:0]", Pass(()));
	
	test("halt -> { x = 0x1234, x       }", Fail(("test", 1, "width")));
	test("halt -> { x = 0x1234, x[15:0] }", Pass(()));
	
	test("+halt",              Fail(("test", 1, "identifier")));
	test("halt",               Fail(("test", 1, "->")));
	test("-> 8'0",             Fail(("test", 1, "empty")));
	test("halt -> 0",          Fail(("test", 1, "width")));
	test("halt -> 0x0",        Fail(("test", 1, "width")));
	test("halt -> 1 + 1",      Fail(("test", 1, "width")));
	test("halt -> 1 + 1[7:0]", Fail(("test", 1, "width")));
	test("halt -> 7'0",        Fail(("test", 1, "multiple")));
	test("halt -> 8'0 8'0",    Fail(("test", 1, "line break")));
	
	test("halt -> 1 == 1 ? 0x12",        Fail(("test", 1, "width")));
	test("halt -> 1 == 1 ? 0x12 : 0x34", Fail(("test", 1, "width")));
	
	test("halt = 0 -> 8'0x12", Fail(("test", 1, "token")));
	test("halt : 0 -> 8'0x12", Fail(("test", 1, "token")));
	
	//test("halt -> (1 == 1)", Fail(("test", 1, "integer")));
}


#[test]
fn test_rules_parameters()
{
	test("load {a} -> 8'0",                    Pass(()));
	test("load {a} -> 8'0 @ a[15:0]",          Pass(()));
	test("load {a} -> 8'0 @ a[15:0] @ a[7:0]", Pass(()));
	
	test("load {a}, {b} -> 8'0",                   Pass(()));
	test("load {a}, {b} -> 8'0 @ a[7:0]",          Pass(()));
	test("load {a}, {b} -> 8'0 @ a[7:0] @ b[7:0]", Pass(()));
	
	test("load +{a}, -{b} -> 8'0 @ a[7:0] @ b[7:0]", Pass(()));
	
	test("load {pc}     -> 8'0", Fail(("test", 1, "reserved")));
	test("load {a}, {a} -> 8'0", Fail(("test", 1, "duplicate")));
	
	test("load {a} -> 8'0 @ a", Fail(("test", 1, "width")));
	
	test("load {a}   {b} -> 8'0", Fail(("test", 1, "separating")));
	test("load {a} + {b} -> 8'0", Fail(("test", 1, "token")));
	
	test("load          -> 8'0 @ a[7:0]", Pass(()));
	test("load {a}, {b} -> 8'0 @ c[7:0]", Pass(()));
}