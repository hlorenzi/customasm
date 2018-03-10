use diagn::RcReport;
use util::FileServer;
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
	
	let report = RcReport::new();
	
	let result = drive_inner(report.clone(), &opts, args, fileserver);
	
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


fn drive_inner(report: RcReport, opts: &getopts::Options, args: &Vec<String>, fileserver: &mut FileServer) -> Result<(), bool>
{
	let matches = parse_opts(report.clone(), opts, args).map_err(|_| true)?;
	
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
	
	if matches.free.len() != 1
		{ return Err(true); }
	
	let asm_file = matches.free[0].clone();
	
	let compiled = compile(report.clone(), fileserver, asm_file, quiet).map_err(|_| false)?;
	
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
		fileserver.write_bytes(report.clone(), &out_data_file, &output_data, None).map_err(|_| false)?;
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


fn parse_opts(report: RcReport, opts: &getopts::Options, args: &Vec<String>) -> Result<getopts::Matches, ()>
{
	match opts.parse(&args[1..])
	{
        Ok(m) => Ok(m),
        Err(f) => Err(report.error(format!("{}", f)))
    }
}


fn print_usage(opts: &getopts::Options)
{
	println!("{}", opts.usage(&format!("Usage: {} [options] <file>", env!("CARGO_PKG_NAME"))));
}


fn print_version()
{
	println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}


fn print_header()
{
	print_version();
}


pub fn compile<S>(report: RcReport, fileserver: &FileServer, asm_file: S, quiet: bool) -> Result<BinaryOutput, ()>
where S: Into<String>
{
	let asm_file_owned = asm_file.into();
	
	if !quiet
	{
		print_header();
		println!("assembling `{}`...", &asm_file_owned);
	}
	
	assemble(report, fileserver, asm_file_owned)
}