use diagn::Report;
use syntax::{tokenize, Parser};
use expr::{Expression, ExpressionValue};
use util::{FileServer, FileServerMock};
use super::ExpectedResult::*;
use super::{ExpectedResult, expect_result};
use num::BigInt;


fn test<S>(src: S, expected: ExpectedResult<ExpressionValue>)
where S: Into<Vec<u8>>
{
	fn compile(report: &mut Report, fileserver: &FileServer) -> Result<ExpressionValue, ()>
	{
		let chars = fileserver.get_chars(report, "test", None)?;
		let tokens = tokenize(report, "test", &chars)?;
		
		let expr = Expression::parse(&mut Parser::new(report, &tokens))?;
		
		expr.check_vars(&mut |_, span| Err(report.error_span("unknown variable", span)))?;
		
		let expr_type = expr.eval_type(report, &|_| panic!("unknown variable"))?;
		let expr_value = expr.eval(report, &|_| panic!("unknown variable"))?;
		
		if !expr_value.is_of_type(expr_type)
			{ panic!("mismatching eval_type and actual result type"); }
			
		Ok(expr_value)
	}
	
	
	let mut report = Report::new();
	let mut fileserver = FileServerMock::new();
	fileserver.add("test", src);
	
	let result = compile(&mut report, &fileserver);
	expect_result(&report, &fileserver, result.ok(), expected);
}


#[test]
fn test_literals()
{
	test("0",  Pass(ExpressionValue::Integer(BigInt::from(0))));
	test("1",  Pass(ExpressionValue::Integer(BigInt::from(1))));
	test("10", Pass(ExpressionValue::Integer(BigInt::from(10))));
	
	test("0b10", Pass(ExpressionValue::Integer(BigInt::from(2))));
	test("0x10", Pass(ExpressionValue::Integer(BigInt::from(16))));
	
	test("0b1_0", Pass(ExpressionValue::Integer(BigInt::from(2))));
	test("0x1_0", Pass(ExpressionValue::Integer(BigInt::from(16))));
	
	test("10a",   Fail(("test", 1, "invalid")));
	test("0b102", Fail(("test", 1, "invalid")));
	test("0b10a", Fail(("test", 1, "invalid")));
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
	
	test("-1 << 0", Pass(ExpressionValue::Integer(BigInt::from(-1))));
	test("-1 << 1", Pass(ExpressionValue::Integer(BigInt::from(-2))));
	test("-1 << 2", Pass(ExpressionValue::Integer(BigInt::from(-4))));
	test("-1 << 3", Pass(ExpressionValue::Integer(BigInt::from(-8))));
	
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
	
	test("  0 @   0", Fail(("test", 1, "concatenation")));
	test("8'0 @   0", Fail(("test", 1, "concatenation")));
	test("  0 @ 8'0", Fail(("test", 1, "concatenation")));
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
	test("(1 == 1) &  (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) &  (1 == 0)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) &  (1 == 1)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) &  (1 == 0)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 1) && (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) && (1 == 0)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) && (1 == 1)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 0) && (1 == 0)", Pass(ExpressionValue::Bool(false)));
	
	test("(1 == 1) |  (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) |  (1 == 0)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) |  (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) |  (1 == 0)", Pass(ExpressionValue::Bool(false)));
	test("(1 == 1) || (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 1) || (1 == 0)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) || (1 == 1)", Pass(ExpressionValue::Bool(true)));
	test("(1 == 0) || (1 == 0)", Pass(ExpressionValue::Bool(false)));
	
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