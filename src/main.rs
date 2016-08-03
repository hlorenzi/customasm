extern crate docopt;


pub mod util;
pub mod rule;
pub mod definition;
pub mod assembler;
pub mod driver;

mod tests;


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


fn main()
{
	let args = docopt::Docopt::new(USAGE)
		.and_then(|d| Ok(d.help(true)))
		.and_then(|d| Ok(d.version(Some(format!("customasm v{}", env!("CARGO_PKG_VERSION"))))))
		.and_then(|d| d.parse())
		.unwrap_or_else(|e| e.exit());
	
	let out_format = match args.get_str("--format")
	{
		"binary" => driver::OutputFormat::Binary,
		"binstr" => driver::OutputFormat::BinStr,
		"hexstr" => driver::OutputFormat::HexStr,
		"bindump" => driver::OutputFormat::BinDump,
		"hexdump" => driver::OutputFormat::HexDump,
		"" => driver::OutputFormat::HexDump,
		_ => util::misc::error_exit(util::error::Error::new("invalid format"))
	};
	
	let out_file = match args.get_str("<out_file>")
	{
		"" => None,
		f => Some(f)
	};
	
	let opt = driver::DriverOptions
	{
		quiet: args.get_bool("--quiet"),
		def_file: args.get_str("<def_file>"),
		asm_file: args.get_str("<asm_file>"),
		out_file: out_file,
		out_format: out_format
	};
	
	driver::driver_main(&opt);
}