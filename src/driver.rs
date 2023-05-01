use crate::*;


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
	DecSpace,
	HexSpace,
	DecC,
	HexC,
	LogiSim8,
	LogiSim16,
	AddressSpan,
}


enum SymbolFormat
{
	Symbols,
	SymbolsMesenMlb,
}


pub fn drive(
	args: &Vec<String>,
	fileserver: &mut dyn util::FileServer)
	-> Result<(), ()>
{
	let mut report = diagn::Report::new();
	
	let result = drive_inner(
		&mut report,
		args,
		fileserver);
	
	if report.has_messages()
		{ println!(""); }
	
	util::enable_windows_ansi_support();
	report.print_all(&mut std::io::stderr(), fileserver);
	
	result
		.map(|_| ())
		.map_err(|_| ())
}


pub fn drive_inner(
	report: &mut diagn::Report,
	args: &Vec<String>,
	fileserver: &mut dyn util::FileServer)
	-> Result<asm::AssemblyResult, ()>
{
	let opts = make_opts();
	
	let matches = parse_opts(
		report,
		&opts,
		args)?;
	
	if matches.opt_present("h")
	{
		print_version_full();
		print_usage(&opts);
		return Ok(asm::AssemblyResult::new());
	}
	
	if matches.opt_present("v")
	{
		print_version_full();
		return Ok(asm::AssemblyResult::new());
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
		Some("decspace")  => OutputFormat::DecSpace,
		Some("hexspace")  => OutputFormat::HexSpace,
		Some("decc")      => OutputFormat::DecC,
		Some("hexc")      => OutputFormat::HexC,
		Some("c")         => OutputFormat::HexC,
		Some("logisim8")  => OutputFormat::LogiSim8,
		Some("logisim16") => OutputFormat::LogiSim16,
		Some("addrspan")  => OutputFormat::AddressSpan,
		
		None => if out_stdout
			{ OutputFormat::AnnotatedHex }
		else
			{ OutputFormat::Binary },
		
		Some(_) =>
		{
			report.error("invalid output format");
			return Err(());
		}
	};

	let symbol_format = match matches.opt_str("symbol-format").as_ref().map(|s| s.as_ref())
	{
		None |
		Some("default")   => SymbolFormat::Symbols,
		Some("mesen-mlb") => SymbolFormat::SymbolsMesenMlb,
		Some(_) =>
		{
			report.error("invalid symbol format");
			return Err(());
		}
	};
	
	if matches.free.len() < 1
	{
		report.error("no input files");
		return Err(());
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
				Some(get_default_output_filename(report, &main_asm_file)?)
			}
		}
	};

	let mut opts = asm::AssemblyOptions::new();
	
	opts.debug_iterations =
		matches.opt_present("debug-iters");

	opts.optimize_statically_known =
		!matches.opt_present("debug-no-optimize-static");

	opts.optimize_instruction_matching =
		!matches.opt_present("debug-no-optimize-matcher");

	if let Some(t) = matches.opt_str("t")
	{
		opts.max_iterations = {
			match t.parse::<usize>()
			{
				Ok(t) => t,
				Err(_) =>
				{
					report.error("invalid number of iterations");
					return Err(());
				}
			}
		};
	}
	
	if !quiet
		{ print_version_short(); }


	let root_filenames = &matches.free;
	
	if !quiet
	{
		for filename in root_filenames
		{
			println!("assembling `{}`...", filename);
		}
	}

	let assembly = asm::assemble(
		report,
		&opts,
		fileserver,
		root_filenames);

	let output = {
		match assembly.output.as_ref()
		{
			Some(o) => o,
			None => return Err(()),
		}
	};

	let decls = assembly.decls.as_ref().unwrap();
	let defs = assembly.defs.as_ref().unwrap();
	let iterations_taken = assembly.iterations_taken.unwrap();


	let output_symbol_data = if output_symbol_file.is_none()
	{
		None
	}
	else
	{
		Some(match symbol_format
		{
			SymbolFormat::Symbols  => decls.symbols.format_default(&decls, &defs),
			SymbolFormat::SymbolsMesenMlb => decls.symbols.format_mesen_mlb(&decls, &defs),
		})
	};

	let output_data: Vec<u8> = match out_format
	{
		OutputFormat::Binary    => output.format_binary(),
		
		OutputFormat::BinStr    => output.format_binstr()           .bytes().collect(),
		OutputFormat::HexStr    => output.format_hexstr()           .bytes().collect(),
		OutputFormat::BinDump   => output.format_bindump()          .bytes().collect(),
		OutputFormat::HexDump   => output.format_hexdump()          .bytes().collect(),
		OutputFormat::Mif       => output.format_mif()              .bytes().collect(),
		OutputFormat::IntelHex  => output.format_intelhex()         .bytes().collect(),
		OutputFormat::DecComma  => output.format_separator(10, ", ").bytes().collect(),
		OutputFormat::HexComma  => output.format_separator(16, ", ").bytes().collect(),
		OutputFormat::DecSpace  => output.format_separator(10, " ") .bytes().collect(),
		OutputFormat::HexSpace  => output.format_separator(16, " ") .bytes().collect(),
		OutputFormat::DecC      => output.format_c_array(10)        .bytes().collect(),
		OutputFormat::HexC      => output.format_c_array(16)        .bytes().collect(),
		OutputFormat::LogiSim8  => output.format_logisim(8)         .bytes().collect(),
		OutputFormat::LogiSim16 => output.format_logisim(16)        .bytes().collect(),
		
		OutputFormat::AnnotatedHex => output.format_annotated_hex(fileserver).bytes().collect(),
		OutputFormat::AnnotatedBin => output.format_annotated_bin(fileserver).bytes().collect(),
		OutputFormat::AddressSpan  => output.format_addrspan     (fileserver).bytes().collect(),
	};
	
	if out_stdout
	{
		if !quiet
		{
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
			fileserver.write_bytes(
				report,
				None,
				&output_file,
				&output_data)?;

			any_files_written = true;
		}

		if let Some(output_symbol_data) = output_symbol_data
		{
			if let Some(ref output_symbol_file) = output_symbol_file
			{
				println!("writing `{}`...", &output_symbol_file);
				fileserver.write_bytes(
					report,
					None,
					&output_symbol_file,
					&output_symbol_data.bytes().collect::<Vec<u8>>())?;

				any_files_written = true;
			}
		}

		if !any_files_written
			{ println!("no files written"); }
	}
	
	if !quiet
	{
		println!(
			"resolved in {} iteration{}",
			iterations_taken,
			if iterations_taken == 1 { "" } else { "s" });
	}
	
	Ok(assembly)
}


fn make_opts() -> getopts::Options
{
    let mut opts = getopts::Options::new();
    opts.optopt("f", "format", "The format of the output file. Possible formats: binary, annotated, annotatedbin, binstr, hexstr, bindump, hexdump, mif, intelhex, deccomma, hexcomma, decspace, hexspace, decc, hexc, logisim8, logisim16, addrspan", "FORMAT");
    opts.opt("o", "output", "The name of the output file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.optopt("", "symbol-format", "The format of the symbol file. Possible formats: default, mesen-mlb", "SYMBOL-FORMAT");
    opts.opt("s", "symbol", "The name of the output symbol file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.opt("t", "iter", "The max number of passes the assembler will attempt (default: 10).", "NUM", getopts::HasArg::Maybe, getopts::Occur::Optional);
	opts.optflag("p", "print", "Print output to stdout instead of writing to a file.");
    opts.optflag("q", "quiet", "Suppress progress reports.");
    opts.optflag("", "debug-iters", "Print debug info for the resolution iterations.");
	opts.optflag("", "debug-no-optimize-static", "Prevent optimization of statically-known values.");
	opts.optflag("", "debug-no-optimize-matcher", "Prevent optimization of the instruction matcher algorithm.");
    opts.optflag("v", "version", "Display version information.");
	opts.optflag("h", "help", "Display this information.");
	
	opts
}


fn parse_opts(
	report: &mut diagn::Report,
	opts: &getopts::Options,
	args: &Vec<String>)
	-> Result<getopts::Matches, ()>
{
	match opts.parse(&args[1..])
	{
        Ok(m) => Ok(m),
        Err(f) =>
		{
			report.error(format!("{}", f));
			Err(())
		}
    }
}


fn print_usage(opts: &getopts::Options)
{
	println!("");
	println!(
		"{}",
		opts.usage(&format!(
			"Usage: {} [options] <asm-file-1> ... <asm-file-N>",
			env!("CARGO_PKG_NAME"))));
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


fn get_default_output_filename(
	report: &mut diagn::Report,
	input_filename: &str)
	-> Result<String, ()>
{
	let mut output_filename = std::path::PathBuf::from(input_filename);
	output_filename.set_extension("bin");
	
	let output_filename = output_filename
		.to_string_lossy()
		.into_owned()
		.replace("\\", "/");
	
	if output_filename == input_filename
	{
		report.error("cannot derive safe output filename");
		return Err(());
	}
	
	Ok(output_filename)
}
