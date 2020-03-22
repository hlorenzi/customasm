use crate::start_flame;
use crate::diagn::RcReport;
use crate::util::FileServer;
use crate::util::enable_windows_ansi_support;
use crate::asm::AssemblerState;
use std::io::{stdout, Read, Write};
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
	
	if matches.free.len() < 1
		{ return Err(true); }
	
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

	let flame_name = matches.opt_str("flame");

	let mut filenames = matches.opt_strs("i");
	for filename in matches.free
		{ filenames.push(filename); }
	
	let mut _flame = start_flame("assemble");
	let assembled = assemble(report.clone(), fileserver, &filenames, quiet).map_err(|_| false)?;

	_flame.drop_start("format output");
	let output = assembled.get_binary_output();
	let output_symbol_data = assembled.get_symbol_output();
	let output_data = match out_format
	{
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
	};

	_flame.drop_start("write output");
	if out_stdout
	{
		if !quiet
		{
			println!("success");
			println!("");
		}
		
		if output_requested || output_file.is_some()
			{ println!("{}", String::from_utf8_lossy(&output_data)); }
			
		if output_symbol_requested || output_symbol_file.is_some()
			{ println!("{}", &output_symbol_data); }
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

		if let Some(ref output_symbol_file) = output_symbol_file
		{
			println!("writing `{}`...", &output_symbol_file);
			fileserver.write_bytes(report.clone(), &output_symbol_file, &output_symbol_data.bytes().collect::<Vec<u8>>(), None).map_err(|_| false)?;
			any_files_written = true;
		}

		if !any_files_written
			{ println!("no files written"); }

		if !quiet
			{ println!("success"); }
	}
	drop(_flame);

    if let Some(filename) = flame_name {
        print!("writing flame `{}`...", &filename);
        std::io::stdout().flush().map_err(|_| false)?;

        if filename.to_lowercase().ends_with("html") {
			let mut html: Vec<u8> = Vec::new();
            flame::dump_html(&mut html).unwrap();
			let mut buf = String::from_utf8_lossy(html.as_slice()).into_owned();
			buf = buf.replacen("width: 100%;", "width: calc(100% - 6px);", 1);
			buf = buf.replacen("margin: 0;", "margin: 0 3px;", 1);
			// buf = buf.replacen("head,", "html { width: 100%; height: 100%; margin: 0; padding: 0; }", 1);
            fileserver.write_bytes(report.clone(), &filename, &buf.bytes().collect::<Vec<u8>>(), None).map_err(|_| false)?;
        } else if filename.to_lowercase().ends_with("json") {
			let mut json: Vec<u8> = Vec::new();
            flame::dump_json(&mut json).unwrap();
            fileserver.write_bytes(report.clone(), &filename, &json, None).map_err(|_| false)?;
        } else {
			let mut text: Vec<u8> = Vec::new();
            flame::dump_text_to_writer(&mut text).unwrap();
            fileserver.write_bytes(report.clone(), &filename, &text, None).map_err(|_| false)?;
        }

        if !quiet {
            println!("success");
        }
    }
	
	Ok(())
}


fn make_opts() -> getopts::Options
{
    let mut opts = getopts::Options::new();
    opts.optopt("f", "format", "The format of the output file. Possible formats: binary, annotated, annotatedbin, binstr, hexstr, bindump, hexdump, mif, intelhex, deccomma, hexcomma, decc, hexc, logisim8, logisim16", "FORMAT");
    opts.optmulti("i", "include", "Specifies an additional file for processing before the given <asm-files>. [deprecated]", "FILE");
    opts.opt("o", "output", "The name of the output file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
	opts.opt("s", "symbol", "The name of the output symbol file.", "FILE", getopts::HasArg::Maybe, getopts::Occur::Optional);
    opts.optopt("", "flame", "The output flame graph. Can export .json, .html, or text (any other extension)", "FLAME");
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


fn get_default_output_filename(report: RcReport, input_filename: &str) -> Result<String, ()>
{
	use std::path::PathBuf;
	
	let mut output_filename = PathBuf::from(input_filename);
	output_filename.set_extension("bin");
	
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