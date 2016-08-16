extern crate customasm;
extern crate docopt;


use customasm::FileHandler;
use std::path::Path;
use std::process::exit;


const USAGE: &'static str = "
Usage:
	customasm [options] <def_file> <asm_file> [<out_file>]
	customasm -v | --version
	customasm -h | --help
	
Options:
	-q, --quiet                     Do not print progress to stdout.
	-f <format>, --format=<format>  The format of the output file. Can be one of:
	                                    binary, binstr, hexstr, bindump, hexdump.
	                                    [default: hexdump]
	-v, --version                   Display version information.
	-h, --help                      Display help.
";


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
	let args = docopt::Docopt::new(USAGE)
		.map(|d| d.help(true))
		.map(|d| d.version(Some(format!("customasm v{}", env!("CARGO_PKG_VERSION")))))
		.and_then(|d| d.parse())
		.unwrap_or_else(|e| e.exit());
	
	let quiet = args.get_bool("--quiet");
	
	let out_format = match args.get_str("--format")
	{
		"binary" =>  OutputFormat::Binary,
		"binstr" =>  OutputFormat::BinStr,
		"hexstr" =>  OutputFormat::HexStr,
		"bindump" => OutputFormat::BinDump,
		"hexdump" => OutputFormat::HexDump,
		_ => error_exit(customasm::Error::new("invalid output format"))
	};
	
	let def_file = Path::new(args.get_str("<def_file>"));
	let asm_file = Path::new(args.get_str("<asm_file>"));
	let out_file = match args.get_str("<out_file>")
	{
		"" => None,
		f => Some(Path::new(f))
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


fn error_exit(err: customasm::Error) -> !
{
	err.print();
	exit(1);
}