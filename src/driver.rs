use diagn::Report;
use util::FileServer;
use instrset::read_instrset;
use asm::BinaryOutput;
use asm::assemble;
use getopts;


enum OutputFormat
{
	Binary,
	BinStr,
	HexStr,
	BinDump,
	HexDump
}


pub fn drive(args: &Vec<String>, fileserver: &mut FileServer) -> Result<(), ()>
{
	let opts = make_opts();
	
	let mut report = Report::new();
	
	let result = drive_inner(&mut report, &opts, args, fileserver);
	
	if report.has_messages()
		{ println!(""); }
	
	report.print_all(fileserver);
	
	if let Err(show_usage) = result
	{
		if show_usage
			{ print_usage(&opts); }
	}
	
	result.map_err(|_| ())
}


fn drive_inner(report: &mut Report, opts: &getopts::Options, args: &Vec<String>, fileserver: &mut FileServer) -> Result<(), bool>
{
	let matches = parse_opts(report, opts, args).map_err(|_| true)?;
	
	if matches.opt_present("h")
	{
		print_usage(&opts);
		return Ok(());
	}
	
	if matches.opt_present("v")
	{
		print_version();
		return Ok(());
	}
	
	let quiet = matches.opt_present("q");
	
	let out_format = match matches.opt_str("f").as_ref().map(|s| s.as_ref())
	{
		Some("binstr")  => OutputFormat::BinStr,
		Some("bindump") => OutputFormat::BinDump,
		Some("hexstr")  => OutputFormat::HexStr,
		Some("hexdump") => OutputFormat::HexDump,
		
		Some("binary") |
		None => OutputFormat::Binary,
		
		Some(_) =>
		{
			report.error("invalid output format");
			return Err(true);
		}
	};
	
	let out_stdout = matches.opt_present("stdout");
	
	let out_data_file = matches.opt_str("o").unwrap_or("a.out".to_string());
	
	if matches.free.len() != 2
		{ return Err(true); }
	
	let instrset_file = matches.free[0].clone();
	let asm_file = matches.free[1].clone();
	
	let compiled = compile(report, fileserver, instrset_file, asm_file, quiet).map_err(|_| false)?;
	
	let output_data = match out_format
	{
		OutputFormat::BinStr  => compiled.generate_binstr (0, compiled.len()).bytes().collect::<Vec<u8>>(),
		OutputFormat::BinDump => compiled.generate_bindump(0, compiled.len()).bytes().collect::<Vec<u8>>(),
		OutputFormat::HexStr  => compiled.generate_hexstr (0, compiled.len()).bytes().collect::<Vec<u8>>(),
		OutputFormat::HexDump => compiled.generate_hexdump(0, compiled.len()).bytes().collect::<Vec<u8>>(),
		OutputFormat::Binary  => compiled.generate_binary (0, compiled.len())
	};
	
	if !quiet
		{ println!("success"); }
	
	if out_stdout
	{
		if !quiet
			{ println!(""); }
			
		println!("{}", String::from_utf8_lossy(&output_data));
	}
	else
	{
		fileserver.write_bytes(report, &out_data_file, &output_data, None).map_err(|_| false)?;
	}
	
	Ok(())
}


fn make_opts() -> getopts::Options
{
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Display this information.");
    opts.optflag("v", "version", "Display version information.");
    opts.optflag("q", "quiet", "Suppress progress reports.");
    opts.optopt("f", "format", "The format of the output file. Possible formats: binary, binstr, hexstr, bindump, hexdump", "FORMAT");
    opts.optopt("o", "out-data", "The name of the output file. (Default: a.out)", "FILE");
    opts.optflag("", "stdout", "Write output to stdout instead of a file.");
	
	opts
}


fn parse_opts(report: &mut Report, opts: &getopts::Options, args: &Vec<String>) -> Result<getopts::Matches, ()>
{
	match opts.parse(&args[1..])
	{
        Ok(m) => Ok(m),
        Err(f) => Err(report.error(format!("{}", f)))
    }
}


fn print_usage(opts: &getopts::Options)
{
	println!("{}", opts.usage(&format!("Usage: {} [options] <instrset-file> <asm-file>", env!("CARGO_PKG_NAME"))));
}


fn print_version()
{
	println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}


fn print_header()
{
	print_version();
}


pub fn compile<S, T>(report: &mut Report, fileserver: &FileServer, instrset_file: S, asm_file: T, quiet: bool) -> Result<BinaryOutput, ()>
where S: Into<String>, T: Into<String>
{
	let instrset_file_owned = instrset_file.into();
	let asm_file_owned = asm_file.into();
	
	if !quiet
	{
		print_header();
		println!("reading `{}`...", &instrset_file_owned);
	}
		
	let instrset = read_instrset(report, fileserver, instrset_file_owned)?;
	
	if !quiet
		{ println!("assembling `{}`...", &asm_file_owned); }
	
	assemble(report, &instrset, fileserver, asm_file_owned)
}