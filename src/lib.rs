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

pub mod asm2;


pub mod webasm;


#[cfg(test)]
pub mod test;


pub fn assemble_str_to_binary(src: &str) -> (Option<Vec<u8>>, diagn::Report)
{
	let mut fileserver = util::FileServerMock::new();
	fileserver.add("str", src.clone());
	
	let assemble = |report: diagn::RcReport, fileserver: &util::FileServerMock, filename: &str| -> Result<Vec<u8>, ()>
	{
		let mut asm = asm::Assembler::new();
		asm.register_file(filename);
		let output = asm.assemble(report.clone(), fileserver, 10)?;
		
		Ok(output.binary.format_binary())
	};
		
	let report = diagn::RcReport::new();
	
	match assemble(report.clone(), &fileserver, "str")
	{
		Ok(output) => (Some(output), report.into_inner()),
		Err(_) => (None, report.into_inner())
	}
}