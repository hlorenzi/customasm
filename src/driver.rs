use definition::Definition;
use translator::translate;
use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;


pub struct DriverOptions<'s>
{
	pub def_file: &'s str,
	pub asm_file: &'s str,
	pub out_file: &'s str,
	pub out_format: OutputFormat
}


pub enum OutputFormat
{
	Binary,
	HexDump
}


pub fn driver_main(opt: &DriverOptions)
{
	println!("parsing definition...");
	
	let def_chars = read_file(opt.def_file);
	let cfg = match Definition::from_src(&def_chars)
	{
		Ok(cfg) => cfg,
		Err(err) =>
		{
			let (line, column) = err.span.get_line_column(&def_chars);
			println!("");
			println!("{}:{}:{}: error: {}", opt.def_file, line, column, err.msg);
			return;
		}
	};
	
	println!("assembling...");
	
	let asm_chars = read_file(opt.asm_file);
	let output = match translate(&cfg, &asm_chars)
	{
		Ok(output) => output,
		Err(err) =>
		{
			let (line, column) = err.span.get_line_column(&asm_chars);
			println!("");
			println!("{}:{}:{}: error: {}", opt.asm_file, line, column, err.msg);
			return;
			
		}
	};
	
	println!("success");
	
	let mut out_file = match File::create(opt.out_file)
	{
		Ok(file) => file,
		Err(err) => error_exit(&format!("{}: error: {}", opt.out_file, err))
	};
	
	match opt.out_format
	{
		OutputFormat::Binary =>
			match out_file.write_all(&output.get_bytes())
			{
				Ok(..) => { }
				Err(err) => error_exit(&format!("{}: error: {}", opt.out_file, err))
			},

		OutputFormat::HexDump =>
			match out_file.write_all(output.get_hex_dump().as_bytes())
			{
				Ok(..) => { }
				Err(err) => error_exit(&format!("{}: error: {}", opt.out_file, err))
			}
	}
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