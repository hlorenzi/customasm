mod expr;


use diagn::Report;
use util::FileServer;
use std::fmt::Debug;
use std::cmp::PartialEq;


type ExpectedResult<T> = Result<T, (&'static str, usize, &'static str)>;


pub fn expect_result<T>(report: &Report, fileserver: &FileServer, got: Option<T>, expected: ExpectedResult<T>)
where T: Debug + PartialEq
{
	report.print_all(fileserver);
	
	if expected.as_ref().ok().is_some()
	{
		if report.has_errors()
		{
			panic!("expected to pass but failed");
		}
		
		if got.is_none()
		{
			panic!("expected to pass but failed");
		}
		
		if got.as_ref().unwrap() != expected.as_ref().ok().unwrap()
		{
			panic!("test failed\nexpected: {:?}\n     got: {:?}", expected.unwrap(), got.unwrap());
		}
	}
	
	else
	{
		if !report.has_errors()
		{
			panic!("expected to fail but passed");
		}
		
		if got.is_some()
		{
			panic!("expected to fail but passed\n     got: {:?}", got.unwrap());
		}
		
		let error = expected.err().unwrap();
		
		if !report.has_error_at(fileserver, error.0, error.1 - 1, error.2)
		{
			panic!("expected a certain error but didn't get it\nexpected: ({:?}, {}, {:?})", error.0, error.1, error.2);
		}
	}
}