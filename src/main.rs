extern crate customasm;
extern crate docopt;


const USAGE: &'static str = "
Usage:
	customasm [--format=<format>] <def_file> <asm_file> <out_file>
	
Options:
	-f <format>, --format=<format>  The format of the output file. Can be one of: binary, hexdump. [Default: binary]
";


fn main()
{
	let args = docopt::Docopt::new(USAGE)
		.and_then(|d| d.parse())
		.unwrap_or_else(|e| e.exit());
	
	let out_format = match args.get_str("--format")
	{
		"binary" => customasm::driver::OutputFormat::Binary,
		"hexdump" => customasm::driver::OutputFormat::HexDump,
		"" => customasm::driver::OutputFormat::Binary,
		_ => customasm::driver::error_exit("invalid format")
	};
	
	let opt = customasm::driver::DriverOptions
	{
		def_file: args.get_str("<def_file>"),
		asm_file: args.get_str("<asm_file>"),
		out_file: args.get_str("<out_file>"),
		out_format: out_format
	};
	
	customasm::driver::driver_main(&opt);
}