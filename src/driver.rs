use crate::asm::BinaryBlock;
use crate::diagn::RcReport;
use crate::util::FileServer;
use crate::util::enable_windows_ansi_support;
use crate::asm::AssemblerState;
use std::io::stdout;
use getopts;


enum OutputFormat
{
	Binary,
	AnnotatedHex,
	AnnotatedBin,
	BinStr,
	HexStr,
	BinDump,
	HexDump,
	Mif,
	IntelHex,
	DecComma,
	HexComma,
	DecC,
	HexC,
	LogiSim8,
	LogiSim16,
}

enum OutputLocation {
	None,
	Stdout,
	File(String)
}

struct OutputFile {
	format: OutputFormat,
	location: OutputLocation
}

impl OutputFile {
	fn new(format_str: &str, location: OutputLocation, report: RcReport) -> Result<OutputFile, bool> {
		let format = match format_str
		{
			"annotated"    => OutputFormat::AnnotatedHex,
			"annotatedhex" => OutputFormat::AnnotatedHex,
			"annotatedbin" => OutputFormat::AnnotatedBin,
			
			"binstr"    => OutputFormat::BinStr,
			"bindump"   => OutputFormat::BinDump,
			"hexstr"    => OutputFormat::HexStr,
			"hexdump"   => OutputFormat::HexDump,
			"binary"    => OutputFormat::Binary,
			"mif"       => OutputFormat::Mif,
			"intelhex"  => OutputFormat::IntelHex,
			"deccomma"  => OutputFormat::DecComma,
			"hexcomma"  => OutputFormat::HexComma,
			"decc"      => OutputFormat::DecC,
			"hexc"      => OutputFormat::HexC,
			"c"         => OutputFormat::HexC,
			"logisim8"  => OutputFormat::LogiSim8,
			"logisim16" => OutputFormat::LogiSim16,
			
			_ =>
			{
				report.error(format!("invalid output format '{}'", format_str));
				return Err(true);
			}
		};

		Ok(OutputFile {
			format, 
			location
		})
	}

	fn generate_output(&self, output: &BinaryBlock, fileserver: &mut dyn FileServer) -> Vec<u8> {
		match self.format {
			OutputFormat::Binary    => output.generate_binary(0, output.len()),
			
			OutputFormat::BinStr    => output.generate_binstr  (0, output.len()).bytes().collect::<Vec<u8>>(),
			OutputFormat::BinDump   => output.generate_bindump (0, output.len()).bytes().collect::<Vec<u8>>(),
			OutputFormat::HexStr    => output.generate_hexstr  (0, output.len()).bytes().collect::<Vec<u8>>(),
			OutputFormat::HexDump   => output.generate_hexdump (0, output.len()).bytes().collect::<Vec<u8>>(),
			OutputFormat::Mif       => output.generate_mif     (0, output.len()).bytes().collect::<Vec<u8>>(),
			OutputFormat::IntelHex  => output.generate_intelhex(0, output.len()).bytes().collect::<Vec<u8>>(),
			OutputFormat::DecComma  => output.generate_comma   (0, output.len(), 10).bytes().collect::<Vec<u8>>(),
			OutputFormat::HexComma  => output.generate_comma   (0, output.len(), 16).bytes().collect::<Vec<u8>>(),
			OutputFormat::DecC      => output.generate_c_array (0, output.len(), 10).bytes().collect::<Vec<u8>>(),
			OutputFormat::HexC      => output.generate_c_array (0, output.len(), 16).bytes().collect::<Vec<u8>>(),
			OutputFormat::LogiSim8  => output.generate_logisim (0, output.len(), 8).bytes().collect::<Vec<u8>>(),
			OutputFormat::LogiSim16 => output.generate_logisim (0, output.len(), 16).bytes().collect::<Vec<u8>>(),
			
			OutputFormat::AnnotatedHex => output.generate_annotated_hex(fileserver, 0, output.len()).bytes().collect::<Vec<u8>>(),
			OutputFormat::AnnotatedBin => output.generate_annotated_bin(fileserver, 0, output.len()).bytes().collect::<Vec<u8>>(),
		}
	}
}

pub fn drive(args: &Vec<String>, fileserver: &mut dyn FileServer) -> Result<(), ()>
{
	let opts = make_opts();
	
	let report = RcReport::new();
	
	let result = drive_inner(report.clone(), &opts, args, fileserver);
	
	if report.has_messages()
		{ println!(""); }
	
	enable_windows_ansi_support();
	report.print_all(&mut stdout(), fileserver);
	
	if let Err(show_usage) = result
	{
		if show_usage
			{ print_usage(&opts); }
	}
	
	result.map_err(|_| ())
}


fn drive_inner(report: RcReport, opts: &getopts::Options, args: &Vec<String>, fileserver: &mut dyn FileServer) -> Result<(), bool>
{
	let matches = parse_opts(report.clone(), opts, args).map_err(|_| true)?;
	
	if matches.opt_present("h")
	{
		print_usage(&opts);
		return Ok(());
	}
	
	if matches.opt_present("v")
	{
		print_info();
		return Ok(());
	}
	
	let quiet = matches.opt_present("q");
	let out_stdout = matches.opt_present("p");

	if matches.free.len() < 1
		{ return Err(true); }
	
	let main_asm_file = matches.free[0].clone();
	
	let output_symbol_location = if !matches.opt_present("s") {
		OutputLocation::None 
	} else if out_stdout {
		OutputLocation::Stdout 
	} else {
		match matches.opt_str("s") {
			Some(f) => OutputLocation::File(f),
			None => match get_default_output_filename(report.clone(), &main_asm_file, "sym")
			{
				Ok(f) => OutputLocation::File(f),
				Err(_) => OutputLocation::None,
			}
		}
	};

	let output_location = if matches.opt_present("s") && !matches.opt_present("o") { 
		// if we specify a symbol file and no output file, we should only output the symbols
		OutputLocation::None 
	} else if out_stdout {
		OutputLocation::Stdout 
	} else {
		match matches.opt_str("o") {
			Some(f) => OutputLocation::File(f),
			None => match get_default_output_filename(report.clone(), &main_asm_file, "bin")
			{
				Ok(f) => OutputLocation::File(f),
				Err(_) => OutputLocation::None,
			}
		}
	};

	let mut output_files: Vec<OutputFile> = vec![];
	if let OutputLocation::None = output_location {} else {
		let format_string = matches.opt_str("f").unwrap_or(if out_stdout { "annotatedhex".to_string() } else { "binary".to_string() });
		output_files.push(OutputFile::new(&format_string, output_location, report.clone())?);
	}

	if !out_stdout {
		for extra in matches.opt_strs("extra").iter() {
			let split: Vec<&str> = extra.splitn(2, ":").collect();
			if split.len() < 2 {
				report.error(format!("extra output definition '{}' must have two components separated by a `:` character", extra));
				return Err(true)
			}
			output_files.push(OutputFile::new(split[0], OutputLocation::File(split[1].to_string()), report.clone())?)
		}
	}

	let mut filenames = matches.opt_strs("i");
	for filename in matches.free
		{ filenames.push(filename); }

	let assembled = assemble(report.clone(), fileserver, &filenames, quiet).map_err(|_| false)?;
	let assembler_output = assembled.get_binary_output();
	let output_symbol_data = assembled.get_symbol_output();

	// when output is directed to stdout, print "success" first. When output is not on stdout, print "success" last.
	if !quiet && out_stdout 
	{
		println!("success");
		println!("");
	}

	let mut any_output = false;
	for output_file in output_files {
		match output_file.location { 
			OutputLocation::Stdout => {
				println!("{}", String::from_utf8_lossy(&output_file.generate_output(&assembler_output, fileserver)));
				any_output = true;
			},
			OutputLocation::File(ref path) => {
				println!("writing `{}`...", &path);
				let output = output_file.generate_output(&assembler_output, fileserver);
				fileserver.write_bytes(report.clone(), &path, &output, None).map_err(|_| false)?;
				any_output = true;
			},
			_ => {}
		}
	}
		
	match output_symbol_location {
		OutputLocation::Stdout => {
			println!("{}", &output_symbol_data);
			any_output = true;
		},
		OutputLocation::File(path) => {
			println!("writing `{}`...", &path);
			fileserver.write_bytes(report.clone(), &path, &output_symbol_data.bytes().collect::<Vec<u8>>(), None).map_err(|_| false)?;
			any_output = true;
		},
		_ => {}
	}

	if !any_output
		{ println!("no output"); }

	if !quiet && !out_stdout
		{ println!("success"); }
	
	Ok(())
}


fn make_opts() -> getopts::Options
{
    let mut opts = getopts::Options::new();
    opts.optopt("f", "format", "The format of the output file. Possible formats: binary, annotated, annotatedbin, binstr, hexstr, bindump, hexdump, mif, intelhex, deccomma, hexcomma, decc, hexc, logisim8, logisim16", "FORMAT");
    opts.optmulti("i", "include", "Specifies an additional file for processing before the given <asm-files>. [deprecated]", "FILE");
    opts.opt("o", "output", "The name of the output file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.opt("s", "symbol", "The name of the output symbol file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.optmulti("", "extra", "Specifies an additional output file and its format", "format:FILE");
    opts.optflag("p", "print", "Print output to stdout instead of writing to a file.");
    opts.optflag("q", "quiet", "Suppress progress reports.");
    opts.optflag("v", "version", "Display version information.");
    opts.optflag("h", "help", "Display this information.");
	
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
	print_info();
	println!("");
	println!("{}", opts.usage(&format!("Usage: {} [options] <asm-file-1> ... <asm-file-N>", env!("CARGO_PKG_NAME"))));
}


fn print_version()
{
	println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}


fn print_info()
{
	print_version();
	println!("https://github.com/hlorenzi/customasm");
}


fn get_default_output_filename(report: RcReport, input_filename: &str, extension: &str) -> Result<String, ()>
{
	use std::path::PathBuf;
	
	let mut output_filename = PathBuf::from(input_filename);
	output_filename.set_extension(extension);
	
	let output_filename = output_filename.to_string_lossy().into_owned().replace("\\", "/");
	
	if output_filename == input_filename
		{ return Err(report.error("cannot derive safe output filename")); }
	
	Ok(output_filename)
}


pub fn assemble(report: RcReport, fileserver: &dyn FileServer, filenames: &[String], quiet: bool) -> Result<AssemblerState, ()>
{
	if !quiet
		{ print_version(); }
	
	let mut asm = AssemblerState::new();
	
	for filename in filenames
	{
		let filename_owned = filename.clone();
		
		if !quiet
			{ println!("assembling `{}`...", &filename_owned); }
	
		asm.process_file(report.clone(), fileserver, filename_owned)?;
	}
	
	asm.wrapup(report)?;
	Ok(asm)
}