extern crate customasm;
extern crate getopts;


use customasm::FileHandler;
use std::path::Path;
use std::process::exit;


enum OutputFormat
{
	Binary,
	BinStr,
	HexStr,
	BinDump,
	HexDump
}


fn main()
{
    let args: Vec<String> = std::env::args().collect();
    
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Display this information.");
    opts.optflag("v", "version", "Display version information.");
    opts.optflag("q", "quiet", "Suppress diagnostics and progress reports.");
    opts.optopt("f", "format", "The format of the output file. Possible formats: binary, binstr, hexstr, bindump, hexdump", "FORMAT");
	
    let matches = match opts.parse(&args[1..])
	{
        Ok(m) => m,
        Err(f) => print_usage(true, &opts, Some(&format!("{}", f)))
    };
	
	if matches.opt_present("h")
	{
		print_usage(false, &opts, None);
	}
	
	if matches.opt_present("v")
	{
		println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		return;
	}
	
	let quiet = matches.opt_present("q");
	
	let out_format = match matches.opt_str("f").as_ref().map(|s| s.as_ref())
	{
		Some("binary") => OutputFormat::Binary,
		Some("binstr") => OutputFormat::BinStr,
		Some("hexstr") => OutputFormat::HexStr,
		Some("bindump") => OutputFormat::BinDump,
		Some("hexdump") |
		None => OutputFormat::HexDump,
		Some(_) => print_usage(true, &opts, Some("Invalid output format."))
	};
	
	let (def_file, asm_file, out_file) =
	{
		if matches.free.len() == 3
		{
			(Path::new(&matches.free[0]),
			Path::new(&matches.free[1]),
			Some(Path::new(&matches.free[2])))
		}
			
		else if matches.free.len() == 2
		{
			(Path::new(&matches.free[0]),
			Path::new(&matches.free[1]),
			None)
		}
		
		else
			{ print_usage(true, &opts, None) }
	};
	
	
	let mut filehandler = customasm::RealFileHandler::new();
	
	if !quiet
		{ println!("reading definition..."); }
	
	let def = match customasm::Definition::from_file(&filehandler, def_file)
	{
		Ok(def) => def,
		Err(err) => error_exit(err)
	};
	
	if !quiet
		{ println!("assembling..."); }
	
	let output_bitvec = match customasm::Assembler::assemble_file(&def, &filehandler, asm_file)
	{
		Ok(output_bitvec) => output_bitvec,
		Err(err) => error_exit(err)
	};
	
	let output = match out_format
	{
		OutputFormat::Binary => output_bitvec.get_bytes(),
		OutputFormat::BinStr => output_bitvec.get_bin_str().as_bytes().to_vec(),
		OutputFormat::HexStr => output_bitvec.get_hex_str().as_bytes().to_vec(),
		OutputFormat::BinDump => output_bitvec.get_bin_dump().as_bytes().to_vec(),
		OutputFormat::HexDump => output_bitvec.get_hex_dump().as_bytes().to_vec()
	};	
	
	match out_file
	{
		Some(f) =>
		{
			match filehandler.write_bytes(f, &output)
			{
				Ok(()) => { }
				Err(err) => error_exit(err)
			}
			
			if !quiet
				{ println!("success"); }
		}
		
		None =>
		{
			if !quiet
			{
				println!("output:");
				println!("");
			}
			
			print!("{}", String::from_utf8_lossy(&output));
			
			if !quiet
				{ println!(""); }
		}
	};
}


fn print_usage(error: bool, opts: &getopts::Options, msg: Option<&str>) -> !
{
	if let Some(msg) = msg
	{
		println!("{}", msg); 
		println!("");
	}
	
	println!("{}", opts.usage(&format!("Usage: customasm [options] <def-file> <asm-file> [<out-file>]")));
	exit(if error { 1 } else { 0 });
}


fn error_exit(err: customasm::Error) -> !
{
	err.print();
	exit(1);
}