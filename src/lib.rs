#![allow(dead_code)]


extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;
extern crate getopts;


pub mod diagn;
pub mod syntax;
pub mod expr;
pub mod asm;
pub mod util;
pub mod driver;


//pub mod webasm;


#[cfg(test)]
pub mod test;


/*pub fn assemble_str_to_binary(src: &str) -> (Option<Vec<u8>>, Report)
{
	let mut fileserver = FileServerMock::new();
	fileserver.add("str", src.clone());
	
	let assemble = |report: diagn::RcReport, fileserver: &FileServerMock, filename: &str| -> Result<Vec<u8>, ()>
	{
		let mut asm = AssemblerState::new();
		asm.process_file(report.clone(), fileserver, filename)?;
		asm.wrapup(report)?;
		
		let output = asm.get_binary_output();
		Ok(output.generate_binary(0, output.len()))
	};
		
	let report = diagn::RcReport::new();
	
	match assemble(report.clone(), &fileserver, "str")
	{
		Ok(output) => (Some(output), report.into_inner()),
		Err(_) => (None, report.into_inner())
	}
}*/