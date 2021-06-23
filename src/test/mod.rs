use crate::*;


mod examples;
mod excerpt;
mod expr;
mod file;
mod lib;


pub enum ExpectedResult<T>
{
	Pass(T),
	Fail((&'static str, usize, &'static str))
}


pub fn expect_result<T>(report: diagn::RcReport, fileserver: &dyn util::FileServer, got: Option<T>, expected: ExpectedResult<T>)
where T: std::fmt::Debug + PartialEq
{
	util::enable_windows_ansi_support();
	
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
		
		if !report.has_first_error_at(fileserver, err.0, err.1 - 1, err.2)
		{
			panic!("expected a certain error but got other errors\nexpected: ({:?}, {}, {:?})", err.0, err.1, err.2);
		}
	}
}