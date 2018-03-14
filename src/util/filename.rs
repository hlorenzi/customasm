use diagn::{Span, RcReport};
use std::path::{PathBuf, Component};


pub fn filename_validate(report: RcReport, f: &str, span: &Span) -> Result<(), ()>
{
	let pathbuf = PathBuf::from(f);
	
	pathbuf.components().fold(Ok(()), |r, c|
		match c
		{
			Component::Prefix(_) |
			Component::RootDir |
			Component::CurDir |
			Component::ParentDir =>
			{
				report.error_span("invalid filename", span);
				r.and(Err(()))
			}
			
			_ => r.and(Ok(()))
		})
}


pub fn filename_navigate(report: RcReport, current: &str, nav: &str, span: &Span) -> Result<String, ()>
{
	if let Err(()) = filename_validate(report, nav, span)
		{ return Err(()); }
		
	let mut result = PathBuf::from(current);
	result.set_file_name(nav);
	
	Ok(result.to_string_lossy().into_owned().replace("\\", "/"))
}