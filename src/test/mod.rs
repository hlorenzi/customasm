mod excerpt;
mod expr;
mod cpudef;
mod asm;


use diagn::RcReport;
use util::FileServer;
use util::enable_windows_ansi_support;
use std::fmt::Debug;
use std::cmp::PartialEq;


pub enum ExpectedResult<T>
{
	Pass(T),
	_Warn(T, (&'static str, usize, &'static str)),
	Fail((&'static str, usize, &'static str))
}


pub fn expect_result<T>(report: RcReport, fileserver: &FileServer, got: Option<T>, expected: ExpectedResult<T>)
where T: Debug + PartialEq
{
	enable_windows_ansi_support();
	
	let mut msgs = Vec::<u8>::new();
	report.print_all(&mut msgs, fileserver);
	print!("{}", String::from_utf8(msgs).unwrap());
	
	if let ExpectedResult::Pass(result) = expected
	{
		if report.has_errors()
		{
			panic!("expected to pass but failed");
		}
		
		if got.is_none()
		{
			panic!("expected to pass but failed");
		}
		
		if let Some(got) = got
		{
			if got != result
				{ panic!("test failed\nexpected: {:?}\n     got: {:?}", result, got); }
		}
	}
	
	else if let ExpectedResult::Fail(err) = expected
	{
		if got.is_some()
		{
			panic!("expected to fail but passed\n     got: {:?}", got.unwrap());
		}
		
		if !report.has_errors()
		{
			panic!("expected to fail but passed");
		}
		
		if !report.has_error_at(fileserver, err.0, err.1 - 1, err.2)
		{
			panic!("expected a certain error but got other errors\nexpected: ({:?}, {}, {:?})", err.0, err.1, err.2);
		}
	}
}