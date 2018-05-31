use diagn::RcReport;
use syntax::{tokenize, Parser};
use expr::{Expression, ExpressionValue, ExpressionEvalContext};
use util::{FileServer, FileServerMock};
use super::ExpectedResult::*;
use super::{ExpectedResult, expect_result};
use num_bigint::BigInt;


fn test<S>(src: S, expected: ExpectedResult<ExpressionValue>)
where S: Into<Vec<u8>>
{
	fn compile(report: RcReport, fileserver: &FileServer) -> Result<ExpressionValue, ()>
	{
		let chars = fileserver.get_chars(report.clone(), "test", None)?;
		let tokens = tokenize(report.clone(), "test", &chars)?;
		
		let expr = Expression::parse(&mut Parser::new(report.clone(), tokens))?;
		
		let expr_value = expr.eval(report.clone(), &mut ExpressionEvalContext::new(),
			&|_, _, _| Err(false),
			&|_, _, _, _| Err(false))?;
		
		Ok(expr_value)
	}
	
	
	let report = RcReport::new();
	let mut fileserver = FileServerMock::new();
	fileserver.add("test", src);
	
	let result = compile(report.clone(), &fileserver);
	expect_result(report.clone(), &fileserver, result.ok(), expected);
}


#[test]
fn test_literals()
{
	test("0",  Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("1",  Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("10", Pass(ExpressionValue::Integer(BigInt::from(10))));
	
	test("0b10", Pass(ExpressionValue::Integer(BigInt::from(2))));
	test("0o10", Pass(ExpressionValue::Integer(BigInt::from(8))));
	test("0x10", Pass(ExpressionValue::Integer(BigInt::from(16))));
	
	test("0b1_0", Pass(ExpressionValue::Integer(BigInt::from(2))));
	test("0x1_0", Pass(ExpressionValue::Integer(BigInt::from(16))));
	
	test("10a",   Fail(("test", 1, "invalid")));
	test("0b102", Fail(("test", 1, "invalid")));
	test("0b10a", Fail(("test", 1, "invalid")));
	test("0o80",  Fail(("test", 1, "invalid")));
	test("0o10a", Fail(("test", 1, "invalid")));
	test("0x10g", Fail(("test", 1, "invalid")));
	
	test("8'0x0'0",  Fail(("test", 1, "invalid")));
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
	test("-0",   Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("-1",   Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-10",  Pass(ExpressionValue::Integer(BigInt::from(-10))));
	test("--10", Pass(ExpressionValue::Integer(BigInt::from(10))));
	
	test("2 + 2",         Pass(ExpressionValue::Integer(BigInt::from(4))));
	test("2 + 2 + 2 + 2", Pass(ExpressionValue::Integer(BigInt::from(8))));
	test("2 - 2",         Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("2 - 2 - 2 - 2", Pass(ExpressionValue::Integer(BigInt::from(-4))));
	test("2 + 2 - 2 + 2", Pass(ExpressionValue::Integer(BigInt::from(4))));
	test("2 * 2",         Pass(ExpressionValue::Integer(BigInt::from(4))));
	test("2 * 2 * 2 * 2", Pass(ExpressionValue::Integer(BigInt::from(16))));
	test("2 / 2",         Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("2 / 2 / 2 / 2", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("2 * 2 * 2 / 2", Pass(ExpressionValue::Integer(BigInt::from(4))));
	test("0 % 2",         Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("1 % 2",         Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("2 % 2",         Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("3 % 2",         Pass(ExpressionValue::Integer(BigInt::from(1))));
	
	test(" 2 * -2", Pass(ExpressionValue::Integer(BigInt::from(-4))));
	test("-2 *  2", Pass(ExpressionValue::Integer(BigInt::from(-4))));
	test("-2 * -2", Pass(ExpressionValue::Integer(BigInt::from(4))));
	
	test(" 2 / -2", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-2 /  2", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-2 / -2", Pass(ExpressionValue::Integer(BigInt::from(1))));
	
	test("1 << 0", Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("1 << 1", Pass(ExpressionValue::Integer(BigInt::from(2))));
	test("1 << 2", Pass(ExpressionValue::Integer(BigInt::from(4))));
	test("1 << 3", Pass(ExpressionValue::Integer(BigInt::from(8))));
	test("1 << 4", Pass(ExpressionValue::Integer(BigInt::from(16))));
	test("1 << 5", Pass(ExpressionValue::Integer(BigInt::from(32))));
	test("1 << 6", Pass(ExpressionValue::Integer(BigInt::from(64))));
	test("1 << 7", Pass(ExpressionValue::Integer(BigInt::from(128))));
	test("1 << 8", Pass(ExpressionValue::Integer(BigInt::from(256))));
	test("1 << 9", Pass(ExpressionValue::Integer(BigInt::from(512))));
	
	test("-1 << 0", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-1 << 1", Pass(ExpressionValue::Integer(BigInt::from(-2))));
	test("-1 << 2", Pass(ExpressionValue::Integer(BigInt::from(-4))));
	test("-1 << 3", Pass(ExpressionValue::Integer(BigInt::from(-8))));
	test("-1 << 4", Pass(ExpressionValue::Integer(BigInt::from(-16))));
	test("-1 << 5", Pass(ExpressionValue::Integer(BigInt::from(-32))));
	test("-1 << 6", Pass(ExpressionValue::Integer(BigInt::from(-64))));
	test("-1 << 7", Pass(ExpressionValue::Integer(BigInt::from(-128))));
	test("-1 << 8", Pass(ExpressionValue::Integer(BigInt::from(-256))));
	test("-1 << 9", Pass(ExpressionValue::Integer(BigInt::from(-512))));
	
	test("4 >> 0", Pass(ExpressionValue::Integer(BigInt::from(4))));
	test("4 >> 1", Pass(ExpressionValue::Integer(BigInt::from(2))));
	test("4 >> 2", Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("4 >> 3", Pass(ExpressionValue::Integer(BigInt::from(0))));
	
	test("-4 >> 0", Pass(ExpressionValue::Integer(BigInt::from(-4))));
	test("-4 >> 1", Pass(ExpressionValue::Integer(BigInt::from(-2))));
	test("-4 >> 2", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-4 >> 3", Pass(ExpressionValue::Integer(BigInt::from(-1))));
}


#[test]
fn test_ops_bitmanipulation()
{
	test("! 0", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("! 1", Pass(ExpressionValue::Integer(BigInt::from(-2))));
	test("!-1", Pass(ExpressionValue::Integer(BigInt::from(0))));
	
	test("0b1100 & 0b1010", Pass(ExpressionValue::Integer(BigInt::from(0b1000))));
	test("0b1100 | 0b1010", Pass(ExpressionValue::Integer(BigInt::from(0b1110))));
	test("0b1100 ^ 0b1010", Pass(ExpressionValue::Integer(BigInt::from(0b0110))));
	
	test(" 1 &  2", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("-1 &  2", Pass(ExpressionValue::Integer(BigInt::from(2))));
	test(" 1 & -2", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("-1 & -2", Pass(ExpressionValue::Integer(BigInt::from(-2))));
	
	test(" 1 |  2", Pass(ExpressionValue::Integer(BigInt::from(3))));
	test("-1 |  2", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test(" 1 | -2", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-1 | -2", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	
	test(" 1 ^  2", Pass(ExpressionValue::Integer(BigInt::from(3))));
	test("-1 ^  2", Pass(ExpressionValue::Integer(BigInt::from(-3))));
	test(" 1 ^ -2", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-1 ^ -2", Pass(ExpressionValue::Integer(BigInt::from(1))));
	
	test(" 0x0001 & 0xffff", Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("-0x0001 & 0xffff", Pass(ExpressionValue::Integer(BigInt::from(0xffff))));
	test(" 0x0001 | 0xffff", Pass(ExpressionValue::Integer(BigInt::from(0xffff))));
	test("-0x0001 | 0xffff", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test(" 0x0001 ^ 0xffff", Pass(ExpressionValue::Integer(BigInt::from(0xfffe))));
	test("-0x0001 ^ 0xffff", Pass(ExpressionValue::Integer(BigInt::from(-0x10000))));
	
	test("0xffff &  0x0001", Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("0xffff & -0x0001", Pass(ExpressionValue::Integer(BigInt::from(0xffff))));
	test("0xffff |  0x0001", Pass(ExpressionValue::Integer(BigInt::from(0xffff))));
	test("0xffff | -0x0001", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("0xffff ^  0x0001", Pass(ExpressionValue::Integer(BigInt::from(0xfffe))));
	test("0xffff ^ -0x0001", Pass(ExpressionValue::Integer(BigInt::from(-0x10000))));
}


#[test]
fn test_ops_slice()
{
	test("8'0x00", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("8'0x0f", Pass(ExpressionValue::Integer(BigInt::from(0xf))));
	test("8'0xff", Pass(ExpressionValue::Integer(BigInt::from(0xff))));
	
	test("0x00[0:0]", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("0x0f[0:0]", Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("0xff[0:0]", Pass(ExpressionValue::Integer(BigInt::from(1))));
	
	test("0x00[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("0x0f[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0xf))));
	test("0xff[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0xff))));
	
	test("0x00[8:1]", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("0x0f[8:1]", Pass(ExpressionValue::Integer(BigInt::from(0x7))));
	test("0xff[8:1]", Pass(ExpressionValue::Integer(BigInt::from(0x7f))));
	
	test("0x12345678[ 3: 0]", Pass(ExpressionValue::Integer(BigInt::from(0x8))));
	test("0x12345678[ 7: 4]", Pass(ExpressionValue::Integer(BigInt::from(0x7))));
	test("0x12345678[11: 8]", Pass(ExpressionValue::Integer(BigInt::from(0x6))));
	test("0x12345678[15:12]", Pass(ExpressionValue::Integer(BigInt::from(0x5))));
	test("0x12345678[19:16]", Pass(ExpressionValue::Integer(BigInt::from(0x4))));
	test("0x12345678[23:20]", Pass(ExpressionValue::Integer(BigInt::from(0x3))));
	test("0x12345678[27:24]", Pass(ExpressionValue::Integer(BigInt::from(0x2))));
	test("0x12345678[31:28]", Pass(ExpressionValue::Integer(BigInt::from(0x1))));
	
	test("-0x01[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0xff))));
	test("-0x08[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0xf8))));
	test("-0x7f[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0x81))));
	test("-0x80[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0x80))));
	test("-0x81[7:0]", Pass(ExpressionValue::Integer(BigInt::from(0x7f))));
	
	test(" 0[1000:1000]", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test(" 1[1000:1000]", Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("-1[1000:1000]", Pass(ExpressionValue::Integer(BigInt::from(1))));
	
	test("0'0x100", Fail(("test", 1, "invalid")));
	test("8'0x100", Fail(("test", 1, "width")));
	
	test("0x00[0:7]", Fail(("test", 1, "invalid")));
	
	test("0x00[0x1_ffff_ffff_ffff_ffff:7]", Fail(("test", 1, "large")));
}


#[test]
fn test_ops_concat()
{
	test("8'0     @ 8'0",     Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("8'0x12  @ 8'0x34",  Pass(ExpressionValue::Integer(BigInt::from(0x1234))));
	test("16'0x12 @ 16'0x34", Pass(ExpressionValue::Integer(BigInt::from(0x120034))));
	
	test("(6 + 6)[3:0] @ (5 + 5)[3:0]", Pass(ExpressionValue::Integer(BigInt::from(0xca))));
	
	test("6'4 @ 5'0", Pass(ExpressionValue::Integer(BigInt::from(0b10000000))));
	
	test("4'0x8 @ 4'0x1", Pass(ExpressionValue::Integer(BigInt::from(0x81))));
	test("4'0x0 @ 4'0x0 @ 4'0x8 @ 4'0x9", Pass(ExpressionValue::Integer(BigInt::from(0x89))));
	test("0x1 @ (4'0x0 @ 4'0x0 @ 4'0x8 @ 4'0x9)", Pass(ExpressionValue::Integer(BigInt::from(0x10089))));
	
	test("  0 @   0", Fail(("test", 1, "known width")));
	test("8'0 @   0", Fail(("test", 1, "known width")));
	test("  0 @ 8'0", Fail(("test", 1, "known width")));
	
	test("-0x1 @  0x1", Fail(("test", 1, "known width")));
	test(" 0x1 @ -0x1", Fail(("test", 1, "known width")));
}


#[test]
fn test_ops_relational_int()
{
	test("0 == 0", Pass(ExpressionValue::Bool(true)));
	test("0 != 0", Pass(ExpressionValue::Bool(false)));
	test("0 == 1", Pass(ExpressionValue::Bool(false)));
	test("0 != 1", Pass(ExpressionValue::Bool(true)));
	
	test("1 <  2", Pass(ExpressionValue::Bool(true)));
	test("1 <= 2", Pass(ExpressionValue::Bool(true)));
	test("2 <  1", Pass(ExpressionValue::Bool(false)));
	test("2 <= 1", Pass(ExpressionValue::Bool(false)));
	test("1 >  2", Pass(ExpressionValue::Bool(false)));
	test("1 >= 2", Pass(ExpressionValue::Bool(false)));
	test("2 >  1", Pass(ExpressionValue::Bool(true)));
	test("2 >= 1", Pass(ExpressionValue::Bool(true)));
	
	test("-1 == -1", Pass(ExpressionValue::Bool(true)));
	test("-1 <  -2", Pass(ExpressionValue::Bool(false)));
	test("-1 <= -2", Pass(ExpressionValue::Bool(false)));
	test("-2 <  -1", Pass(ExpressionValue::Bool(true)));
	test("-2 <= -1", Pass(ExpressionValue::Bool(true)));
	test("-1 >  -2", Pass(ExpressionValue::Bool(true)));
	test("-1 >= -2", Pass(ExpressionValue::Bool(true)));
	test("-2 >  -1", Pass(ExpressionValue::Bool(false)));
	test("-2 >= -1", Pass(ExpressionValue::Bool(false)));
	
	test("2 <  2", Pass(ExpressionValue::Bool(false)));
	test("2 <= 2", Pass(ExpressionValue::Bool(true)));
	test("2 >  2", Pass(ExpressionValue::Bool(false)));
	test("2 >= 2", Pass(ExpressionValue::Bool(true)));
	
	test(" !(1 == 1)", Pass(ExpressionValue::Bool(false)));
	test(" !(1 != 1)", Pass(ExpressionValue::Bool(true)));
	test("!!(1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("!!(1 != 1)", Pass(ExpressionValue::Bool(false)));
}


#[test]
fn test_ops_relational_bool()
{
	test("(1 == 1) & (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) & (1 == 0)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) & (1 == 1)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) & (1 == 0)", Pass(ExpressionValue::Bool(false)));
	
	test("(1 == 1) | (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) | (1 == 0)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) | (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) | (1 == 0)", Pass(ExpressionValue::Bool(false)));
	
	test("(1 == 1) ^ (1 == 1)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 1) ^ (1 == 0)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) ^ (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) ^ (1 == 0)", Pass(ExpressionValue::Bool(false)));
	
	test("(1 == 1) == (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) == (1 == 0)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) == (1 == 1)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) == (1 == 0)", Pass(ExpressionValue::Bool(true)));
	
	test("(1 == 1) != (1 == 1)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 1) != (1 == 0)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) != (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) != (1 == 0)", Pass(ExpressionValue::Bool(false)));
}


#[test]
fn test_ops_lazy()
{
	test("(1 == 1) && (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) && (1 == 0)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) && (1 == 1)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) && (1 == 0)", Pass(ExpressionValue::Bool(false)));
	
	test("(1 == 1) || (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) || (1 == 0)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) || (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) || (1 == 0)", Pass(ExpressionValue::Bool(false)));
	
	test("(1 == 1) && 123",     Fail(("test", 1, "type")));
	test("(1 == 1) && (1 / 0)", Fail(("test", 1, "zero")));
	test("(1 == 0) && 123",     Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) && (1 / 0)", Pass(ExpressionValue::Bool(false)));
	
	test("(1 == 1) || 123",     Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) || (1 / 0)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) || 123",     Fail(("test", 1, "type")));
	test("(1 == 0) || (1 / 0)", Fail(("test", 1, "zero")));
}


#[test]
fn test_ops_ternary()
{
	test("(1 == 1) ? 123", Pass(ExpressionValue::Integer(BigInt::from(123))));
	test("(1 == 0) ? 123", Pass(ExpressionValue::Void));
	
	test("(1 == 1) ? 123 : 456", Pass(ExpressionValue::Integer(BigInt::from(123))));
	test("(1 == 0) ? 123 : 456", Pass(ExpressionValue::Integer(BigInt::from(456))));
	
	test("(1 == 1) ? 123 : (1 == 1)", Pass(ExpressionValue::Integer(BigInt::from(123))));
	test("(1 == 0) ? 123 : (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) ? (1 == 1) : 123", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) ? (1 == 1) : 123", Pass(ExpressionValue::Integer(BigInt::from(123))));
	
	test("(1 == 1) ? 123 : (1 / 0)", Pass(ExpressionValue::Integer(BigInt::from(123))));
	test("(1 == 0) ? 123 : (1 / 0)", Fail(("test", 1, "zero")));
	test("(1 == 1) ? (1 / 0) : 123", Fail(("test", 1, "zero")));
	test("(1 == 0) ? (1 / 0) : 123", Pass(ExpressionValue::Integer(BigInt::from(123))));
	
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
	test(" 2 +  2  * 2 ", Pass(ExpressionValue::Integer(BigInt::from(6))));
	test(" 2 + (2  * 2)", Pass(ExpressionValue::Integer(BigInt::from(6))));
	test("(2 +  2) * 2 ", Pass(ExpressionValue::Integer(BigInt::from(8))));
	
	test("1 + 2 == 2 + 1", Pass(ExpressionValue::Bool(true)));
	test("1 + 2 != 2 + 1", Pass(ExpressionValue::Bool(false)));
	test("1 + 2 >  2 + 1", Pass(ExpressionValue::Bool(false)));
	test("1 + 2 >= 2 + 1", Pass(ExpressionValue::Bool(true)));
	test("1 + 2 <  2 + 1", Pass(ExpressionValue::Bool(false)));
	test("1 + 2 <= 2 + 1", Pass(ExpressionValue::Bool(true)));
	
	test("0b110 == 0b110 && 0b11 == 0b11", Pass(ExpressionValue::Bool(true)));
	test("0b110 == 0b110 &  0b11 == 0b11", Fail(("test", 1, "argument")));
	test("0b110 == 0b110 || 0b11 == 0b11", Pass(ExpressionValue::Bool(true)));
	test("0b110 == 0b110 |  0b11 == 0b11", Fail(("test", 1, "argument")));
}


#[test]
fn test_blocks()
{
	test("{}",       Pass(ExpressionValue::Void));
	test("{0}",      Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("{0,}",     Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("{0 \n }",  Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("{0,\n }",  Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("{0,   1}", Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("{0 \n 1}", Pass(ExpressionValue::Integer(BigInt::from(1))));
	
	test("{0 1}", Fail(("test", 1, "`,`")));
	
	test("{1 + 2, 3} + {4, 5 + 6}", Pass(ExpressionValue::Integer(BigInt::from(14))));
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
	test("   x  = 123",      Pass(ExpressionValue::Void));
	test("{  x  = 123, x }", Pass(ExpressionValue::Integer(BigInt::from(123))));
	test("{ (x) = 123, x }", Pass(ExpressionValue::Integer(BigInt::from(123))));
	
	test("{ x = 123, y = 321,            x + y }",       Pass(ExpressionValue::Integer(BigInt::from(444))));
	test("{ x = 123, y = x * 2,          x + y }",       Pass(ExpressionValue::Integer(BigInt::from(369))));
	test("{ x = 123, y = x * 2, x = 753, x + y }",       Pass(ExpressionValue::Integer(BigInt::from(999))));
	
	test("{ x = 123, y = 456, x < y ? min = x : min = y, min }", Pass(ExpressionValue::Integer(BigInt::from(123))));
	test("{ x = 456, y = 123, x < y ? min = x : min = y, min }", Pass(ExpressionValue::Integer(BigInt::from(123))));
	
	test("{ x = 123, y = 456, x < y ? min = x,           min }", Pass(ExpressionValue::Integer(BigInt::from(123))));
	test("{ x = 456, y = 123, x < y ? min = x,           min }", Fail(("test", 1, "unknown")));
	
	test("0 = 1",     Fail(("test", 1, "invalid")));
	test("x + 1 = 2", Fail(("test", 1, "invalid")));
	test("{x} = 1",   Fail(("test", 1, "invalid")));
}