use crate::*;
use super::ExpectedResult::*;
use super::{ExpectedResult, expect_result};


fn test<S>(src: S, expected: ExpectedResult<expr::Value>)
where S: Into<Vec<u8>>
{
	fn compile(report: diagn::RcReport, fileserver: &dyn util::FileServer) -> Result<expr::Value, ()>
	{
		let chars = fileserver.get_chars(report.clone(), "test", None)?;
		let tokens = syntax::tokenize(report.clone(), "test", &chars)?;
		
		let expr = expr::Expr::parse(
			&mut syntax::Parser::new(Some(report.clone()), &tokens))?;
		
		let expr_value = expr.eval(
			report.clone(),
			&mut expr::EvalContext::new(),
			&|_| Err(false),
			&|_| Err(false),
			&|_| Err(()))?;
		
		Ok(expr_value)
	}
	
	
	let report = diagn::RcReport::new();
	let mut fileserver = util::FileServerMock::new();
	fileserver.add("test", src);
	
	let result = compile(report.clone(), &fileserver);
	expect_result(report.clone(), &fileserver, result.ok(), expected);
}


#[test]
fn test_literals()
{
	test("0",  Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("1",  Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("10", Pass(expr::Value::make_integer(util::BigInt::new(10, None))));
	
	test("0b10", Pass(expr::Value::make_integer(util::BigInt::new(2, Some(2)))));
	test("0o10", Pass(expr::Value::make_integer(util::BigInt::new(8, Some(6)))));
	test("0x10", Pass(expr::Value::make_integer(util::BigInt::new(16, Some(8)))));
	
	test("0b1_0", Pass(expr::Value::make_integer(util::BigInt::new(2, Some(2)))));
	test("0x1_0", Pass(expr::Value::make_integer(util::BigInt::new(16, Some(8)))));
	
	test("0x", Fail(("test", 1, "invalid")));
	
	test("10a",   Fail(("test", 1, "invalid")));
	test("0b102", Fail(("test", 1, "invalid")));
	test("0b10a", Fail(("test", 1, "invalid")));
	test("0o80",  Fail(("test", 1, "invalid")));
	test("0o10a", Fail(("test", 1, "invalid")));
	test("0x10g", Fail(("test", 1, "invalid")));
	
	test("8'0x0",    Fail(("test", 1, "invalid")));
	test("0b8'0x00", Fail(("test", 1, "invalid")));
	test("0x8'0x00", Fail(("test", 1, "invalid")));
}


#[test]
fn test_variables()
{
	test(" a", Fail(("test", 1, "unknown")));
	test(".a", Fail(("test", 1, "unknown")));
	
	test("1 +  a + 1", Fail(("test", 1, "unknown")));
	test("1 + .a + 1", Fail(("test", 1, "unknown")));
}


#[test]
fn test_ops_arithmetic()
{
	test("-0",   Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("-1",   Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("-10",  Pass(expr::Value::make_integer(util::BigInt::new(-10, None))));
	test("--10", Pass(expr::Value::make_integer(util::BigInt::new(10, None))));
	
	test("2 + 2",         Pass(expr::Value::make_integer(util::BigInt::new(4, None))));
	test("2 + 2 + 2 + 2", Pass(expr::Value::make_integer(util::BigInt::new(8, None))));
	test("2 - 2",         Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("2 - 2 - 2 - 2", Pass(expr::Value::make_integer(util::BigInt::new(-4, None))));
	test("2 + 2 - 2 + 2", Pass(expr::Value::make_integer(util::BigInt::new(4, None))));
	test("2 * 2",         Pass(expr::Value::make_integer(util::BigInt::new(4, None))));
	test("2 * 2 * 2 * 2", Pass(expr::Value::make_integer(util::BigInt::new(16, None))));
	test("2 / 2",         Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("2 / 2 / 2 / 2", Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("2 * 2 * 2 / 2", Pass(expr::Value::make_integer(util::BigInt::new(4, None))));
	test("0 % 2",         Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("1 % 2",         Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("2 % 2",         Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("3 % 2",         Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	
	test(" 2 * -2", Pass(expr::Value::make_integer(util::BigInt::new(-4, None))));
	test("-2 *  2", Pass(expr::Value::make_integer(util::BigInt::new(-4, None))));
	test("-2 * -2", Pass(expr::Value::make_integer(util::BigInt::new(4, None))));
	
	test(" 2 / -2", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("-2 /  2", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("-2 / -2", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	
	test("1 << 0", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("1 << 1", Pass(expr::Value::make_integer(util::BigInt::new(2, None))));
	test("1 << 2", Pass(expr::Value::make_integer(util::BigInt::new(4, None))));
	test("1 << 3", Pass(expr::Value::make_integer(util::BigInt::new(8, None))));
	test("1 << 4", Pass(expr::Value::make_integer(util::BigInt::new(16, None))));
	test("1 << 5", Pass(expr::Value::make_integer(util::BigInt::new(32, None))));
	test("1 << 6", Pass(expr::Value::make_integer(util::BigInt::new(64, None))));
	test("1 << 7", Pass(expr::Value::make_integer(util::BigInt::new(128, None))));
	test("1 << 8", Pass(expr::Value::make_integer(util::BigInt::new(256, None))));
	test("1 << 9", Pass(expr::Value::make_integer(util::BigInt::new(512, None))));
	
	test("-1 << 0", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("-1 << 1", Pass(expr::Value::make_integer(util::BigInt::new(-2, None))));
	test("-1 << 2", Pass(expr::Value::make_integer(util::BigInt::new(-4, None))));
	test("-1 << 3", Pass(expr::Value::make_integer(util::BigInt::new(-8, None))));
	test("-1 << 4", Pass(expr::Value::make_integer(util::BigInt::new(-16, None))));
	test("-1 << 5", Pass(expr::Value::make_integer(util::BigInt::new(-32, None))));
	test("-1 << 6", Pass(expr::Value::make_integer(util::BigInt::new(-64, None))));
	test("-1 << 7", Pass(expr::Value::make_integer(util::BigInt::new(-128, None))));
	test("-1 << 8", Pass(expr::Value::make_integer(util::BigInt::new(-256, None))));
	test("-1 << 9", Pass(expr::Value::make_integer(util::BigInt::new(-512, None))));
	
	test("4 >> 0", Pass(expr::Value::make_integer(util::BigInt::new(4, None))));
	test("4 >> 1", Pass(expr::Value::make_integer(util::BigInt::new(2, None))));
	test("4 >> 2", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("4 >> 3", Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	
	test("-4 >> 0", Pass(expr::Value::make_integer(util::BigInt::new(-4, None))));
	test("-4 >> 1", Pass(expr::Value::make_integer(util::BigInt::new(-2, None))));
	test("-4 >> 2", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("-4 >> 3", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	
	test("123`0 + 2", Pass(expr::Value::make_integer(util::BigInt::new(2, None))));
}


#[test]
fn test_ops_bitmanipulation()
{
	test("! 0", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("! 1", Pass(expr::Value::make_integer(util::BigInt::new(-2, None))));
	test("!-1", Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	
	test("!7", Pass(expr::Value::make_integer(util::BigInt::new(-8, None))));
	test("!8", Pass(expr::Value::make_integer(util::BigInt::new(-9, None))));
	test("!9", Pass(expr::Value::make_integer(util::BigInt::new(-10, None))));
	test("!16", Pass(expr::Value::make_integer(util::BigInt::new(-17, None))));
	test("!32", Pass(expr::Value::make_integer(util::BigInt::new(-33, None))));
	test("!64", Pass(expr::Value::make_integer(util::BigInt::new(-65, None))));
	test("!128", Pass(expr::Value::make_integer(util::BigInt::new(-129, None))));
	test("!256", Pass(expr::Value::make_integer(util::BigInt::new(-257, None))));

	test("!-8", Pass(expr::Value::make_integer(util::BigInt::new(7, None))));
	test("!-9", Pass(expr::Value::make_integer(util::BigInt::new(8, None))));
	test("!-10", Pass(expr::Value::make_integer(util::BigInt::new(9, None))));
	test("!-17", Pass(expr::Value::make_integer(util::BigInt::new(16, None))));
	test("!-33", Pass(expr::Value::make_integer(util::BigInt::new(32, None))));
	test("!-65", Pass(expr::Value::make_integer(util::BigInt::new(64, None))));
	test("!-129", Pass(expr::Value::make_integer(util::BigInt::new(128, None))));
	test("!-257", Pass(expr::Value::make_integer(util::BigInt::new(256, None))));
	
	test("0b1100 & 0b1010", Pass(expr::Value::make_integer(util::BigInt::new(0b1000, None))));
	test("0b1100 | 0b1010", Pass(expr::Value::make_integer(util::BigInt::new(0b1110, None))));
	test("0b1100 ^ 0b1010", Pass(expr::Value::make_integer(util::BigInt::new(0b0110, None))));
	
	test(" 1 &  2", Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("-1 &  2", Pass(expr::Value::make_integer(util::BigInt::new(2, None))));
	test(" 1 & -2", Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("-1 & -2", Pass(expr::Value::make_integer(util::BigInt::new(-2, None))));
	
	test(" 1 |  2", Pass(expr::Value::make_integer(util::BigInt::new(3, None))));
	test("-1 |  2", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test(" 1 | -2", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("-1 | -2", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	
	test(" 1 ^  2", Pass(expr::Value::make_integer(util::BigInt::new(3, None))));
	test("-1 ^  2", Pass(expr::Value::make_integer(util::BigInt::new(-3, None))));
	test(" 1 ^ -2", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("-1 ^ -2", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	
	test(" 0x0001 & 0xffff", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("-0x0001 & 0xffff", Pass(expr::Value::make_integer(util::BigInt::new(0xffff, None))));
	test(" 0x0001 | 0xffff", Pass(expr::Value::make_integer(util::BigInt::new(0xffff, None))));
	test("-0x0001 | 0xffff", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test(" 0x0001 ^ 0xffff", Pass(expr::Value::make_integer(util::BigInt::new(0xfffe, None))));
	test("-0x0001 ^ 0xffff", Pass(expr::Value::make_integer(util::BigInt::new(-0x10000, None))));
	
	test("0xffff &  0x0001", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("0xffff & -0x0001", Pass(expr::Value::make_integer(util::BigInt::new(0xffff, None))));
	test("0xffff |  0x0001", Pass(expr::Value::make_integer(util::BigInt::new(0xffff, None))));
	test("0xffff | -0x0001", Pass(expr::Value::make_integer(util::BigInt::new(-1, None))));
	test("0xffff ^  0x0001", Pass(expr::Value::make_integer(util::BigInt::new(0xfffe, None))));
	test("0xffff ^ -0x0001", Pass(expr::Value::make_integer(util::BigInt::new(-0x10000, None))));
}


#[test]
fn test_ops_slice()
{
	test("0x00`8",  Pass(expr::Value::make_integer(util::BigInt::new(0, Some(8)))));
	test("0x0f`8",  Pass(expr::Value::make_integer(util::BigInt::new(0xf, Some(8)))));
	test("0xff`8",  Pass(expr::Value::make_integer(util::BigInt::new(0xff, Some(8)))));
	test("0x101`8", Pass(expr::Value::make_integer(util::BigInt::new(0x1, Some(8)))));
	
	test("0`0",  Pass(expr::Value::make_integer(util::BigInt::new(0, Some(0)))));
	test("0x0`0",  Pass(expr::Value::make_integer(util::BigInt::new(0, Some(0)))));
	test("0x100`0",  Pass(expr::Value::make_integer(util::BigInt::new(0, Some(0)))));
	test("0xfff`0",  Pass(expr::Value::make_integer(util::BigInt::new(0, Some(0)))));

	test("0x00[0:0]", Pass(expr::Value::make_integer(util::BigInt::new(0, Some(1)))));
	test("0x0f[0:0]", Pass(expr::Value::make_integer(util::BigInt::new(1, Some(1)))));
	test("0xff[0:0]", Pass(expr::Value::make_integer(util::BigInt::new(1, Some(1)))));
	
	test("0x00[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0, Some(8)))));
	test("0x0f[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0xf, Some(8)))));
	test("0xff[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0xff, Some(8)))));
	
	test("0x00[8:1]", Pass(expr::Value::make_integer(util::BigInt::new(0, Some(8)))));
	test("0x0f[8:1]", Pass(expr::Value::make_integer(util::BigInt::new(0x7, Some(8)))));
	test("0xff[8:1]", Pass(expr::Value::make_integer(util::BigInt::new(0x7f, Some(8)))));
	
	test("0x12345678[ 3: 0]", Pass(expr::Value::make_integer(util::BigInt::new(0x8, Some(4)))));
	test("0x12345678[ 7: 4]", Pass(expr::Value::make_integer(util::BigInt::new(0x7, Some(4)))));
	test("0x12345678[11: 8]", Pass(expr::Value::make_integer(util::BigInt::new(0x6, Some(4)))));
	test("0x12345678[15:12]", Pass(expr::Value::make_integer(util::BigInt::new(0x5, Some(4)))));
	test("0x12345678[19:16]", Pass(expr::Value::make_integer(util::BigInt::new(0x4, Some(4)))));
	test("0x12345678[23:20]", Pass(expr::Value::make_integer(util::BigInt::new(0x3, Some(4)))));
	test("0x12345678[27:24]", Pass(expr::Value::make_integer(util::BigInt::new(0x2, Some(4)))));
	test("0x12345678[31:28]", Pass(expr::Value::make_integer(util::BigInt::new(0x1, Some(4)))));
	
	test("-0x01[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0xff, Some(8)))));
	test("-0x08[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0xf8, Some(8)))));
	test("-0x7f[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0x81, Some(8)))));
	test("-0x80[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0x80, Some(8)))));
	test("-0x81[7:0]", Pass(expr::Value::make_integer(util::BigInt::new(0x7f, Some(8)))));
	
	test(" 0[1000:1000]", Pass(expr::Value::make_integer(util::BigInt::new(0, Some(1)))));
	test(" 1[1000:1000]", Pass(expr::Value::make_integer(util::BigInt::new(0, Some(1)))));
	test("-1[1000:1000]", Pass(expr::Value::make_integer(util::BigInt::new(1, Some(1)))));
	
	test("0x00[0:7]", Fail(("test", 1, "invalid")));
	
	test("0x00[0x1_ffff_ffff_ffff_ffff:7]", Fail(("test", 1, "large")));
}


#[test]
fn test_ops_concat()
{
	test("0`8     @ 0`8",     Pass(expr::Value::make_integer(util::BigInt::new(0, Some(16)))));
	test("0x12`8  @ 0x34`8",  Pass(expr::Value::make_integer(util::BigInt::new(0x1234, Some(16)))));
	test("0x12`16 @ 0x34`16", Pass(expr::Value::make_integer(util::BigInt::new(0x120034, Some(32)))));
	
	test("0`8     @ 0`0 @ 0`8",     Pass(expr::Value::make_integer(util::BigInt::new(0, Some(16)))));
	test("0x12`8  @ 0`0 @ 0x34`8",  Pass(expr::Value::make_integer(util::BigInt::new(0x1234, Some(16)))));
	test("0x12`16 @ 0`0 @ 0x34`16", Pass(expr::Value::make_integer(util::BigInt::new(0x120034, Some(32)))));

	test("(6 + 6)[3:0] @ (5 + 5)[3:0]", Pass(expr::Value::make_integer(util::BigInt::new(0xca, Some(8)))));
	
	test("4`6 @ 0`5", Pass(expr::Value::make_integer(util::BigInt::new(0b10000000, Some(11)))));
	
	test("0x8`4 @ 0x1`4", Pass(expr::Value::make_integer(util::BigInt::new(0x81, Some(8)))));
	test("0x0`4 @ 0x0`4 @ 0x8`4 @ 0x9`4", Pass(expr::Value::make_integer(util::BigInt::new(0x89, Some(16)))));
	test("0x1 @ (0x0`4 @ 0x0`4 @ 0x8`4 @ 0x9`4)", Pass(expr::Value::make_integer(util::BigInt::new(0x10089, Some(20)))));
	
	test("0x0 @ 0x8000[19:0]", Pass(expr::Value::make_integer(util::BigInt::new(0x8000, Some(24)))));
	test("0x1 @ 0x8000[19:0]", Pass(expr::Value::make_integer(util::BigInt::new(0x108000, Some(24)))));
	test("0x1 @ 0x9000[19:0]", Pass(expr::Value::make_integer(util::BigInt::new(0x109000, Some(24)))));
	
	test("0   @ 0", Fail(("test", 1, "unspecified size")));
	test("0`8 @ 0", Fail(("test", 1, "unspecified size")));
	test("0   @ 0`8", Fail(("test", 1, "unspecified size")));
	
	test("-0x1 @  0x1", Fail(("test", 1, "unspecified size")));
	test(" 0x1 @ -0x1", Fail(("test", 1, "unspecified size")));
}


#[test]
fn test_ops_relational_int()
{
	test("0 == 0", Pass(expr::Value::Bool(true)));
	test("0 != 0", Pass(expr::Value::Bool(false)));
	test("0 == 1", Pass(expr::Value::Bool(false)));
	test("0 != 1", Pass(expr::Value::Bool(true)));
	
	test("(0xff & 0xff) == 0xff", Pass(expr::Value::Bool(true)));
	test("(0xff & 0xff) != 0xff", Pass(expr::Value::Bool(false)));
	test("(0xff & 0xff) == 0xee", Pass(expr::Value::Bool(false)));
	test("(0xff & 0xff) != 0xee", Pass(expr::Value::Bool(true)));

	test("1 <  2", Pass(expr::Value::Bool(true)));
	test("1 <= 2", Pass(expr::Value::Bool(true)));
	test("2 <  1", Pass(expr::Value::Bool(false)));
	test("2 <= 1", Pass(expr::Value::Bool(false)));
	test("1 >  2", Pass(expr::Value::Bool(false)));
	test("1 >= 2", Pass(expr::Value::Bool(false)));
	test("2 >  1", Pass(expr::Value::Bool(true)));
	test("2 >= 1", Pass(expr::Value::Bool(true)));
	
	test("-1 == -1", Pass(expr::Value::Bool(true)));
	test("-1 <  -2", Pass(expr::Value::Bool(false)));
	test("-1 <= -2", Pass(expr::Value::Bool(false)));
	test("-2 <  -1", Pass(expr::Value::Bool(true)));
	test("-2 <= -1", Pass(expr::Value::Bool(true)));
	test("-1 >  -2", Pass(expr::Value::Bool(true)));
	test("-1 >= -2", Pass(expr::Value::Bool(true)));
	test("-2 >  -1", Pass(expr::Value::Bool(false)));
	test("-2 >= -1", Pass(expr::Value::Bool(false)));
	
	test("2 <  2", Pass(expr::Value::Bool(false)));
	test("2 <= 2", Pass(expr::Value::Bool(true)));
	test("2 >  2", Pass(expr::Value::Bool(false)));
	test("2 >= 2", Pass(expr::Value::Bool(true)));
	
	test(" !(1 == 1)", Pass(expr::Value::Bool(false)));
	test(" !(1 != 1)", Pass(expr::Value::Bool(true)));
	test("!!(1 == 1)", Pass(expr::Value::Bool(true)));
	test("!!(1 != 1)", Pass(expr::Value::Bool(false)));
}


#[test]
fn test_ops_relational_bool()
{
	test("(1 == 1) & (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 1) & (1 == 0)", Pass(expr::Value::Bool(false)));
	test("(1 == 0) & (1 == 1)", Pass(expr::Value::Bool(false)));
	test("(1 == 0) & (1 == 0)", Pass(expr::Value::Bool(false)));
	
	test("(1 == 1) | (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 1) | (1 == 0)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) | (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) | (1 == 0)", Pass(expr::Value::Bool(false)));
	
	test("(1 == 1) ^ (1 == 1)", Pass(expr::Value::Bool(false)));
	test("(1 == 1) ^ (1 == 0)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) ^ (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) ^ (1 == 0)", Pass(expr::Value::Bool(false)));
	
	test("(1 == 1) == (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 1) == (1 == 0)", Pass(expr::Value::Bool(false)));
	test("(1 == 0) == (1 == 1)", Pass(expr::Value::Bool(false)));
	test("(1 == 0) == (1 == 0)", Pass(expr::Value::Bool(true)));
	
	test("(1 == 1) != (1 == 1)", Pass(expr::Value::Bool(false)));
	test("(1 == 1) != (1 == 0)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) != (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) != (1 == 0)", Pass(expr::Value::Bool(false)));
}


#[test]
fn test_ops_lazy()
{
	test("(1 == 1) && (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 1) && (1 == 0)", Pass(expr::Value::Bool(false)));
	test("(1 == 0) && (1 == 1)", Pass(expr::Value::Bool(false)));
	test("(1 == 0) && (1 == 0)", Pass(expr::Value::Bool(false)));
	
	test("(1 == 1) || (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 1) || (1 == 0)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) || (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) || (1 == 0)", Pass(expr::Value::Bool(false)));
	
	test("(1 == 1) && 123",     Fail(("test", 1, "type")));
	test("(1 == 1) && (1 / 0)", Fail(("test", 1, "zero")));
	test("(1 == 0) && 123",     Pass(expr::Value::Bool(false)));
	test("(1 == 0) && (1 / 0)", Pass(expr::Value::Bool(false)));
	
	test("(1 == 1) || 123",     Pass(expr::Value::Bool(true)));
	test("(1 == 1) || (1 / 0)", Pass(expr::Value::Bool(true)));
	test("(1 == 0) || 123",     Fail(("test", 1, "type")));
	test("(1 == 0) || (1 / 0)", Fail(("test", 1, "zero")));
}


#[test]
fn test_ops_ternary()
{
	test("(1 == 1) ? 123", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	test("(1 == 0) ? 123", Pass(expr::Value::Void));
	
	test("(1 == 1) ? 123 : 456", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	test("(1 == 0) ? 123 : 456", Pass(expr::Value::make_integer(util::BigInt::new(456, None))));
	
	test("(1 == 1) ? 123 : (1 == 1)", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	test("(1 == 0) ? 123 : (1 == 1)", Pass(expr::Value::Bool(true)));
	test("(1 == 1) ? (1 == 1) : 123", Pass(expr::Value::Bool(true)));
	test("(1 == 0) ? (1 == 1) : 123", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	
	test("(1 == 1) ? 123 : (1 / 0)", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	test("(1 == 0) ? 123 : (1 / 0)", Fail(("test", 1, "zero")));
	test("(1 == 1) ? (1 / 0) : 123", Fail(("test", 1, "zero")));
	test("(1 == 0) ? (1 / 0) : 123", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	
	test("123 ? 456 : 789", Fail(("test", 1, "type")));
}


#[test]
fn test_ops_arith_errors()
{
	test("2 / 0",       Fail(("test", 1, "division by zero")));
	test("2 / (1 - 1)", Fail(("test", 1, "division by zero")));
	test("2 % 0",       Fail(("test", 1, "modulo by zero")));
	test("2 % (1 - 1)", Fail(("test", 1, "modulo by zero")));
	
	test("2 << (1 << 1000)", Fail(("test", 1, "invalid shift")));
	test("2 >> (1 << 1000)", Fail(("test", 1, "invalid shift")));
	test("2 >> (1 << 1000)", Fail(("test", 1, "invalid shift")));
}


#[test]
fn test_ops_type_errors()
{
	test("(1 == 1) + (1 == 1)", Fail(("test", 1, "argument")));
	
	test("-(1 == 1)", Fail(("test", 1, "argument")));
	
	test("(1 == 1) @ (1 == 1)", Fail(("test", 1, "argument")));
}


#[test]
fn test_precedence()
{
	test(" 2 +  2  * 2 ", Pass(expr::Value::make_integer(util::BigInt::new(6, None))));
	test(" 2 + (2  * 2)", Pass(expr::Value::make_integer(util::BigInt::new(6, None))));
	test("(2 +  2) * 2 ", Pass(expr::Value::make_integer(util::BigInt::new(8, None))));
	
	test("1 + 2 == 2 + 1", Pass(expr::Value::Bool(true)));
	test("1 + 2 != 2 + 1", Pass(expr::Value::Bool(false)));
	test("1 + 2 >  2 + 1", Pass(expr::Value::Bool(false)));
	test("1 + 2 >= 2 + 1", Pass(expr::Value::Bool(true)));
	test("1 + 2 <  2 + 1", Pass(expr::Value::Bool(false)));
	test("1 + 2 <= 2 + 1", Pass(expr::Value::Bool(true)));
	
	test("0b110 == 0b110 && 0b11 == 0b11", Pass(expr::Value::Bool(true)));
	test("0b110 == 0b110 &  0b11 == 0b11", Fail(("test", 1, "argument")));
	test("0b110 == 0b110 || 0b11 == 0b11", Pass(expr::Value::Bool(true)));
	test("0b110 == 0b110 |  0b11 == 0b11", Fail(("test", 1, "argument")));
}


#[test]
fn test_blocks()
{
	test("{}",       Pass(expr::Value::Void));
	test("{0}",      Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("{0,}",     Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("{0 \n }",  Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("{0,\n }",  Pass(expr::Value::make_integer(util::BigInt::new(0, None))));
	test("{0,   1}", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	test("{0 \n 1}", Pass(expr::Value::make_integer(util::BigInt::new(1, None))));
	
	test("{0 1}", Fail(("test", 1, "`,`")));
	
	test("{1 + 2, 3} + {4, 5 + 6}", Pass(expr::Value::make_integer(util::BigInt::new(14, None))));
}


#[test]
fn test_calls()
{
	test("0()",        Fail(("test", 1, "callable")));
	test("0(1, 2, 3)", Fail(("test", 1, "callable")));
}


#[test]
fn test_assignment()
{
	test("   x  = 123",      Pass(expr::Value::Void));
	test("{  x  = 123, x }", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	test("{ (x) = 123, x }", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	
	test("{ x = 123, y = 321,            x + y }",       Pass(expr::Value::make_integer(util::BigInt::new(444, None))));
	test("{ x = 123, y = x * 2,          x + y }",       Pass(expr::Value::make_integer(util::BigInt::new(369, None))));
	test("{ x = 123, y = x * 2, x = 753, x + y }",       Pass(expr::Value::make_integer(util::BigInt::new(999, None))));
	
	test("{ x = 123, y = 456, x < y ? min = x : min = y, min }", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	test("{ x = 456, y = 123, x < y ? min = x : min = y, min }", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	
	test("{ x = 123, y = 456, x < y ? min = x,           min }", Pass(expr::Value::make_integer(util::BigInt::new(123, None))));
	test("{ x = 456, y = 123, x < y ? min = x,           min }", Fail(("test", 1, "unknown")));
	
	test("0 = 1",     Fail(("test", 1, "invalid")));
	test("x + 1 = 2", Fail(("test", 1, "invalid")));
	test("{x} = 1",   Fail(("test", 1, "invalid")));
}