use definition;
use assembler;
use std::iter::Iterator;
use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;


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
	
	let def_chars = read_file(opt.def_file);
	let def = match definition::parse(&def_chars)
	{
		Ok(def) => def,
		Err(err) =>
		{
			let (line, column) = err.span.get_line_column(&def_chars);
			println!("");
			println!("{}:{}:{}: error: {}", opt.def_file, line, column, err.msg);
			return;
		}
	};
	
	if !opt.quiet
		{ println!("assembling..."); }
	
	let asm_chars = read_file(opt.asm_file);
	let output_bitvec = match assembler::assemble(&def, &asm_chars)
	{
		Ok(output_bitvec) => output_bitvec,
		Err(err) =>
		{
			let (line, column) = err.span.get_line_column(&asm_chars);
			println!("");
			println!("{}:{}:{}: error: {}", opt.asm_file, line, column, err.msg);
			return;
		}
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
				Err(err) => error_exit(&format!("{}: error: {}", filename, err))
			};
			match out_file.write_all(&output)
			{
				Ok(..) => { }
				Err(err) => error_exit(&format!("{}: error: {}", filename, err))
			}
			if !opt.quiet
				{ println!("success"); }
		}
		
		None =>
		{
			if !opt.quiet
				{ println!("output:"); }
			
			print!("{}", String::from_utf8_lossy(&output))
		}
	};
}


pub fn error_exit(msg: &str) -> !
{
	println!("{}", msg);
	exit(1);
}


fn read_file(filename: &str) -> Vec<char>
{
	let mut file = match File::open(filename)
	{
		Ok(file) => file,
		Err(err) => error_exit(&format!("{}: error: {}", filename, err))
	};

	let mut s = String::new();
	match file.read_to_string(&mut s)
	{
		Ok(..) => s.chars().collect::<Vec<char>>(),
		Err(err) => error_exit(&format!("{}: error: {}", filename, err))
	}
}