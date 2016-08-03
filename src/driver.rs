use definition;
use assembler;
use util::misc;
use util::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;


pub struct DriverOptions<'s>
{
	pub quiet: bool,
	pub def_file: &'s str,
	pub asm_file: &'s str,
	pub out_file: Option<&'s str>,
	pub out_format: OutputFormat
}


pub enum OutputFormat
{
	Binary,
	BinStr,
	HexStr,
	BinDump,
	HexDump
}


pub fn driver_main(opt: &DriverOptions)
{
	if !opt.quiet
		{ println!("reading definition..."); }
	
	let def_chars = misc::read_file(&PathBuf::from(opt.def_file));
	let def = match definition::parse(&opt.def_file, &def_chars)
	{
		Ok(def) => def,
		Err(err) => misc::error_exit(err)
	};
	
	if !opt.quiet
		{ println!("assembling..."); }
	
	let asm_chars = misc::read_file(&PathBuf::from(opt.asm_file));
	let output_bitvec = match assembler::assemble(&def, &opt.asm_file, &asm_chars)
	{
		Ok(output_bitvec) => output_bitvec,
		Err(err) => misc::error_exit(err)
	};
	
	let output = match opt.out_format
	{
		OutputFormat::Binary => output_bitvec.get_bytes(),
		OutputFormat::BinStr => output_bitvec.get_bin_str().as_bytes().to_vec(),
		OutputFormat::HexStr => output_bitvec.get_hex_str().as_bytes().to_vec(),
		OutputFormat::BinDump => output_bitvec.get_bin_dump().as_bytes().to_vec(),
		OutputFormat::HexDump => output_bitvec.get_hex_dump().as_bytes().to_vec()
	};	
	
	match opt.out_file
	{
		Some(filename) =>
		{
			let mut out_file = match File::create(filename)
			{
				Ok(file) => file,
				Err(err) => misc::error_exit(Error::new_with_file(filename, format!("{}", err)))
			};
			
			match out_file.write_all(&output)
			{
				Ok(..) => { }
				Err(err) => misc::error_exit(Error::new_with_file(filename, format!("{}", err)))
			};
			
			if !opt.quiet
				{ println!("success"); }
		}
		
		None =>
		{
			if !opt.quiet
			{
				println!("output:");
				println!("");
			}
			
			print!("{}", String::from_utf8_lossy(&output));
			
			if !opt.quiet
				{ println!(""); }
		}
	};
}