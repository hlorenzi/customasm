use diagn::{Span, RcReport};
use syntax::excerpt_as_string_contents;
use util::FileServerMock;
use super::ExpectedResult::*;
use super::{ExpectedResult, expect_result};
use std::rc::Rc;


fn test(src: &str, expected: ExpectedResult<&str>)
{
	let mut src_quoted = String::new();
	src_quoted.push_str("\"");
	src_quoted.push_str(src);
	src_quoted.push_str("\"");
	let src_quoted: &str = src_quoted.as_ref();

	let report = RcReport::new();
	let mut fileserver = FileServerMock::new();
	fileserver.add("test", src_quoted);
	
	let span = Span::new(Rc::new("test".to_string()), 0, 0);
	
	let result = excerpt_as_string_contents(report.clone(), src_quoted, &span).ok();
	let result = result.as_ref().map(|s| s.as_ref());
	expect_result(report.clone(), &fileserver, result, expected);
}


#[test]
fn test_escape_sequences()
{
	test("",      Pass(""));
	test("hello", Pass("hello"));
	test("áéíóú", Pass("áéíóú"));
	
	test("\\0\\t\\r\\n", Pass("\0\t\r\n"));
	test("\\\\\\'\\\"",  Pass("\\\'\""));
	
	test("\\x00", Pass("\x00"));
	test("\\x01", Pass("\x01"));
	test("\\x0a", Pass("\x0a"));
	test("\\x0A", Pass("\x0a"));
	test("\\x10", Pass("\x10"));
	test("\\x7f", Pass("\x7f"));
	test("\\x80", Fail(("test", 1, "invalid")));
	test("\\xff", Fail(("test", 1, "invalid")));
	test("\\x",   Fail(("test", 1, "invalid")));
	test("\\x0",  Fail(("test", 1, "invalid")));
	test("\\xf",  Fail(("test", 1, "invalid")));
	test("\\x0g", Fail(("test", 1, "invalid")));
	
	test("\\u{}",       Pass("\u{0}"));
	test("\\u{0}",      Pass("\u{0}"));
	test("\\u{e}",      Pass("\u{e}"));
	test("\\u{00}",     Pass("\u{00}"));
	test("\\u{ab}",     Pass("\u{ab}"));
	test("\\u{000}",    Pass("\u{000}"));
	test("\\u{abc}",    Pass("\u{abc}"));
	test("\\u{0000}",   Pass("\u{0000}"));
	test("\\u{abcd}",   Pass("\u{abcd}"));
	test("\\u{00000}",  Pass("\u{00000}"));
	test("\\u{abcde}",  Pass("\u{abcde}"));
	test("\\u{000000}", Pass("\u{000000}"));
	test("\\u{10ffff}", Pass("\u{10ffff}"));
	test("\\u{110000}", Fail(("test", 1, "invalid")));
	test("\\u",         Fail(("test", 1, "invalid")));
	test("\\u{",        Fail(("test", 1, "invalid")));
	test("\\u{0",       Fail(("test", 1, "invalid")));
	test("\\u{0g}",     Fail(("test", 1, "invalid")));
}