use crate::*;
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


enum SymbolFormat
{
	Default,
	MesenMlb,
}


pub fn drive(args: &Vec<String>, fileserver: &mut dyn util::FileServer) -> Result<(), ()>
{
	let opts = make_opts();
	
	let report = diagn::RcReport::new();
	
	let result = drive_inner(report.clone(), &opts, args, fileserver);
	
	if report.has_messages()
		{ println!(""); }
	
	util::enable_windows_ansi_support();
	report.print_all(&mut std::io::stderr(), fileserver);
	
	if let Err(show_usage) = result
	{
		if show_usage
		{
			print_version_short();
			print_usage(&opts);
		}
	}
	
	result.map_err(|_| ())
}


fn drive_inner(
	report: diagn::RcReport,
	opts: &getopts::Options,
	args: &Vec<String>,
	fileserver: &mut dyn util::FileServer)
	-> Result<(), bool>
{
	let matches = parse_opts(report.clone(), opts, args).map_err(|_| true)?;
	
	if matches.opt_present("h")
	{
		print_version_full();
		print_usage(&opts);
		return Ok(());
	}
	
	if matches.opt_present("v")
	{
		print_version_full();
		return Ok(());
	}
	
	let quiet = matches.opt_present("q");
	let out_stdout = matches.opt_present("p");
	
	let out_format = match matches.opt_str("f").as_ref().map(|s| s.as_ref())
	{
		Some("annotated")    => OutputFormat::AnnotatedHex,
		Some("annotatedhex") => OutputFormat::AnnotatedHex,
		Some("annotatedbin") => OutputFormat::AnnotatedBin,
		
		Some("binstr")    => OutputFormat::BinStr,
		Some("bindump")   => OutputFormat::BinDump,
		Some("hexstr")    => OutputFormat::HexStr,
		Some("hexdump")   => OutputFormat::HexDump,
		Some("binary")    => OutputFormat::Binary,
		Some("mif")       => OutputFormat::Mif,
		Some("intelhex")  => OutputFormat::IntelHex,
		Some("deccomma")  => OutputFormat::DecComma,
		Some("hexcomma")  => OutputFormat::HexComma,
		Some("decc")      => OutputFormat::DecC,
		Some("hexc")      => OutputFormat::HexC,
		Some("c")         => OutputFormat::HexC,
		Some("logisim8")  => OutputFormat::LogiSim8,
		Some("logisim16") => OutputFormat::LogiSim16,
		
		None => if out_stdout
			{ OutputFormat::AnnotatedHex }
		else
			{ OutputFormat::Binary },
		
		Some(_) =>
		{
			report.error("invalid output format");
			return Err(true);
		}
	};

	let symbol_format = match matches.opt_str("symbol-format").as_ref().map(|s| s.as_ref())
	{
		None |
		Some("default")   => SymbolFormat::Default,
		Some("mesen-mlb") => SymbolFormat::MesenMlb,
		Some(_) =>
		{
			report.error("invalid symbol format");
			return Err(true);
		}
	};
	
	if matches.free.len() < 1
	{
		report.error("no input files");
		return Err(true);
	}
	
	let main_asm_file = matches.free[0].clone();
	
	let output_symbol_requested = matches.opt_present("s");
	let output_requested = matches.opt_present("o");

	let output_symbol_file = matches.opt_str("s");
	let output_file = match matches.opt_str("o")
	{
		Some(f) => Some(f),
		None =>
		{
			if output_symbol_requested || output_symbol_file.is_some()
				{ None }
			else
			{
				match get_default_output_filename(report.clone(), &main_asm_file)
				{
					Ok(f) => Some(f),
					Err(_) => None,
				}
			}
		}
	};

	let max_iterations = match matches.opt_str("t")
	{
		None => 10,
		Some(t) =>
		{
			match t.parse::<usize>()
			{
				Ok(t) => t,
				Err(_) =>
				{
					report.error("invalid number of iterations");
					return Err(true);
				}
			}
		}
	};
	
	if !quiet
		{ print_version_short(); }
	
	let mut assembler = asm::Assembler::new();
	for filename in matches.free
	{
		if !quiet
		{
			println!("assembling `{}`...", filename);
		}
		
		assembler.register_file(filename);
	}

	let output = assembler.assemble(
		report.clone(),
		fileserver,
		max_iterations)
		.map_err(|_| false)?;

	let binary = output.binary;

	let output_symbol_data = if output_symbol_file.is_none()
	{
		None
	}
	else
	{
		Some(match symbol_format
		{
			SymbolFormat::Default  => output.state.symbols.format_default(),
			SymbolFormat::MesenMlb => output.state.symbols.format_mesen_mlb(&output.state),
		})
	};

	let output_data: Vec<u8> = match out_format
	{
		OutputFormat::Binary    => binary.format_binary(),
		
		OutputFormat::BinStr    => binary.format_binstr  ()  .bytes().collect(),
		OutputFormat::HexStr    => binary.format_hexstr  ()  .bytes().collect(),
		OutputFormat::BinDump   => binary.format_bindump ()  .bytes().collect(),
		OutputFormat::HexDump   => binary.format_hexdump ()  .bytes().collect(),
		OutputFormat::Mif       => binary.format_mif     ()  .bytes().collect(),
		OutputFormat::IntelHex  => binary.format_intelhex()  .bytes().collect(),
		OutputFormat::DecComma  => binary.format_comma   (10).bytes().collect(),
		OutputFormat::HexComma  => binary.format_comma   (16).bytes().collect(),
		OutputFormat::DecC      => binary.format_c_array (10).bytes().collect(),
		OutputFormat::HexC      => binary.format_c_array (16).bytes().collect(),
		OutputFormat::LogiSim8  => binary.format_logisim (8) .bytes().collect(),
		OutputFormat::LogiSim16 => binary.format_logisim (16).bytes().collect(),
		
		OutputFormat::AnnotatedHex => binary.format_annotated_hex(fileserver).bytes().collect(),
		OutputFormat::AnnotatedBin => binary.format_annotated_bin(fileserver).bytes().collect(),
	};
	
	if out_stdout
	{
		if !quiet
		{
			println!("success after {} iteration{}",
				output.iterations,
				if output.iterations == 1 { "" } else { "s" });
			println!("");
		}
		
		if output_requested || output_file.is_some()
			{ println!("{}", String::from_utf8_lossy(&output_data)); }
			
		if output_symbol_requested || output_symbol_file.is_some()
		{
			if let Some(output_symbol_data) = output_symbol_data
				{ println!("{}", &output_symbol_data); }
		}
	}
	else
	{
		let mut any_files_written = false;

		if let Some(ref output_file) = output_file
		{
			println!("writing `{}`...", &output_file);
			fileserver.write_bytes(report.clone(), &output_file, &output_data, None).map_err(|_| false)?;
			any_files_written = true;
		}

		if let Some(output_symbol_data) = output_symbol_data
		{
			if let Some(ref output_symbol_file) = output_symbol_file
			{
				println!("writing `{}`...", &output_symbol_file);
				fileserver.write_bytes(report.clone(), &output_symbol_file, &output_symbol_data.bytes().collect::<Vec<u8>>(), None).map_err(|_| false)?;
				any_files_written = true;
			}
		}

		if !any_files_written
			{ println!("no files written"); }

		if !quiet
		{
			println!("success after {} iteration{}",
				output.iterations,
				if output.iterations == 1 { "" } else { "s" });
		}
	}
	
	Ok(())
}


fn make_opts() -> getopts::Options
{
    let mut opts = getopts::Options::new();
    opts.optopt("f", "format", "The format of the output file. Possible formats: binary, annotated, annotatedbin, binstr, hexstr, bindump, hexdump, mif, intelhex, deccomma, hexcomma, decc, hexc, logisim8, logisim16", "FORMAT");
    opts.opt("o", "output", "The name of the output file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.optopt("", "symbol-format", "The format of the symbol file. Possible formats: default, mesen-mlb", "SYMBOL-FORMAT");
    opts.opt("s", "symbol", "The name of the output symbol file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.opt("t", "iter", "The max number of passes the assembler will attempt (default: 10).", "NUM", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.optflag("p", "print", "Print output to stdout instead of writing to a file.");
    opts.optflag("q", "quiet", "Suppress progress reports.");
    opts.optflag("v", "version", "Display version information.");
	opts.optflag("h", "help", "Display this information.");
	
	opts
}


fn parse_opts(report: diagn::RcReport, opts: &getopts::Options, args: &Vec<String>) -> Result<getopts::Matches, ()>
{
	match opts.parse(&args[1..])
	{
        Ok(m) => Ok(m),
        Err(f) => Err(report.error(format!("{}", f)))
    }
}


fn print_usage(opts: &getopts::Options)
{
	println!("");
	println!("{}", opts.usage(&format!("Usage: {} [options] <asm-file-1> ... <asm-file-N>", env!("CARGO_PKG_NAME"))));
}


fn print_version_short()
{
	let mut version = env!("VERGEN_SEMVER_LIGHTWEIGHT").to_string();
	if version == "UNKNOWN"
	{
		version = format!("v{}", env!("CARGO_PKG_VERSION"));
	}


	let mut date = format!("{}, ", env!("VERGEN_COMMIT_DATE"));
	if date == "UNKNOWN, "
	{
		date = "".to_string();
	}


	println!("{} {} ({}{})",
		env!("CARGO_PKG_NAME"),
		version,
		date,
		env!("VERGEN_TARGET_TRIPLE"));
}


fn print_version_full()
{
	print_version_short();
	println!("https://github.com/hlorenzi/customasm");
}


fn get_default_output_filename(report: diagn::RcReport, input_filename: &str) -> Result<String, ()>
{
	use std::path::PathBuf;
	
	let mut output_filename = PathBuf::from(input_filename);
	output_filename.set_extension("bin");
	
	let output_filename = output_filename.to_string_lossy().into_owned().replace("\\", "/");
	
	if output_filename == input_filename
		{ return Err(report.error("cannot derive safe output filename")); }
	
	Ok(output_filename)
}
