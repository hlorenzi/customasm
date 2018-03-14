mod excerpt;
mod expr;
mod instrset;
mod asm;


use diagn::RcReport;
use util::FileServer;
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
	report.print_all(fileserver);
	
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