use crate::*;


struct Command
{
	pub input_filenames: Vec<String>,
	pub output_groups: Vec<CommandOutput>,
	pub opts: asm::AssemblyOptions,
	pub quiet: bool,
	pub use_colors: bool,
	pub show_version: bool,
	pub show_help: bool,
}


struct CommandOutput
{
	pub format: Option<OutputFormat>,
	pub printout: bool,
	pub output_filename: Option<String>,
}


pub enum OutputFormat
{
	Binary,
	Annotated {
		base: usize,
		group: usize,
	},
	BinStr,
	HexStr,
	BinDump,
	HexDump,
	Mif,
	IntelHex {
		address_unit: usize,
	},
	List(util::FormatListOptions),
	DecComma,
	HexComma,
	DecSpace,
	HexSpace,
	DecC,
	HexC,
	LogiSim8,
	LogiSim16,
	AddressSpan,
	TCGame {
		base: usize,
		group: usize,
	},

	Symbols,
	SymbolsMesenMlb,
}


pub fn drive_from_commandline(
	args: &Vec<String>,
	fileserver: &mut dyn util::FileServer)
	-> Result<(), ()>
{
	util::enable_windows_ansi_support();

	let mut report = diagn::Report::new();

	let maybe_command = parse_command(
		&mut report,
		args);

	if let Ok(command) = maybe_command
	{
		let maybe_result = assemble_with_command(
			&mut report,
			fileserver,
			&command);

		report.print_all(
			&mut std::io::stderr(),
			fileserver,
			command.use_colors);

		maybe_result.map(|_| ())
	}
	else
	{
		report.print_all(
			&mut std::io::stderr(),
			fileserver,
			true);

		Err(())
	}
}


pub fn drive(
	report: &mut diagn::Report,
	args: &Vec<String>,
	fileserver: &mut dyn util::FileServer)
	-> Result<asm::AssemblyResult, ()>
{
	let command = parse_command(
		report,
		args)?;

	let maybe_result = assemble_with_command(
		report,
		fileserver,
		&command);

	maybe_result
}


fn assemble_with_command(
	report: &mut diagn::Report,
	fileserver: &mut dyn util::FileServer,
	command: &Command)
	-> Result<asm::AssemblyResult, ()>
{
	if command.show_help
	{
		print_usage(command.use_colors);
		return Ok(asm::AssemblyResult::new());
	}

	if command.show_version
	{
		print_version_full();
		return Ok(asm::AssemblyResult::new());
	}

	if command.input_filenames.len() < 1
	{
		report.error("no input files");
		return Err(());
	}

	if !command.quiet
	{
		print_version_short();

		for filename in &command.input_filenames
		{
			println!("assembling `{}`...", filename);
		}
	}

	let assembly = asm::assemble(
		report,
		&command.opts,
		fileserver,
		&command.input_filenames);

	let output = assembly.output
		.as_ref()
		.ok_or(())?;

	let decls = assembly.decls.as_ref().unwrap();
	let defs = assembly.defs.as_ref().unwrap();
	let iterations_taken = assembly.iterations_taken.unwrap();

	for output_group in &command.output_groups
	{
		if let Some(format) = &output_group.format
		{
			let formatted = format_output(
				fileserver,
				decls,
				defs,
				output,
				format);

			if output_group.printout
			{
				if !command.quiet
				{
					println!("");
				}

				println!(
					"{}",
					String::from_utf8_lossy(&formatted));
			}
			else if let Some(ref output_filename) = output_group.output_filename
			{
				if !command.quiet
				{
					println!("writing `{}`...", &output_filename);
				}

				fileserver.write_bytes(
					report,
					None,
					&output_filename,
					&formatted)?;
			}
		}
	}

	if !command.quiet
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
	let asm_opts = asm::AssemblyOptions::new();

    let mut opts = getopts::Options::new();

    opts.optopt(
		"f", "format",
		"The format of the output file.\n\
		See below for possible values.",
		"FORMAT");

	opts.opt(
		"o", "output",
		"The name of the output file.",
		"FILE",
		getopts::HasArg::Maybe,
		getopts::Occur::Optional);

    opts.opt(
		"t", "iters",
		&format!(
			"The max number of resolution iterations to attempt. (Default: {})",
			asm_opts.max_iterations),
		"NUM",
		getopts::HasArg::Maybe,
		getopts::Occur::Optional);

	opts.optflag(
		"p", "print",
		"Print the output to the screen instead of writing to a file.");

    opts.optflag(
		"q", "quiet",
		"Suppress progress reports.");

	opts.opt(
		"d", "define",
		"Defines a constant.",
		"VALUE",
		getopts::HasArg::Yes,
		getopts::Occur::Multi);

	opts.opt(
		"", "color",
		"Style the output with colors. [on/off]",
		"VALUE",
		getopts::HasArg::Maybe,
		getopts::Occur::Optional);

    opts.optflag(
		"", "debug-iters",
		"Print debug info for the resolution iterations.");

	opts.optflag(
		"", "debug-no-optimize-static",
		"Prevent optimization of statically-known values.");

	opts.optflag(
		"", "debug-no-optimize-matcher",
		"Prevent optimization of the instruction matcher algorithm.");

    opts.optflag(
		"v", "version",
		"Display version information.");

	opts.optflag(
		"h", "help",
		"Display this information.");

	opts
}


fn parse_command(
	report: &mut diagn::Report,
	args: &Vec<String>)
	-> Result<Command, ()>
{
	let args_groups = args[1..]
		.split(|arg| arg == "--")
		.collect::<Vec<_>>();

	let mut command = Command {
		input_filenames: Vec::new(),
		output_groups: Vec::new(),
		opts: asm::AssemblyOptions::new(),
		quiet: false,
		use_colors: true,
		show_version: false,
		show_help: false,
	};

	let parse_opts = make_opts();

	for arg_group in args_groups
	{
		let parsed = {
			match parse_opts.parse(arg_group)
			{
				Ok(parsed) => parsed,
				Err(failure) =>
				{
					report.error(format!("{}", failure));
					return Err(());
				}
			}
		};

		let mut group = CommandOutput {
			format: None,
			output_filename: None,
			printout: false,
		};


		// Parse options for current output group
		if let Some(format_str) = parsed.opt_str("f").as_ref()
		{
			group.format = Some(parse_output_format(
				report,
				&format_str)?);
		}

		if let Some(output_filename) = parsed.opt_str("o").as_ref()
		{
			group.output_filename = Some(
				output_filename.clone());
		}

		group.printout |= parsed.opt_present("p");


		// Parse global command options
		command.quiet |= parsed.opt_present("q");
		command.show_version |= parsed.opt_present("v");
		command.show_help |= parsed.opt_present("h");

		for define_arg in parsed.opt_strs("d")
		{
			command.opts.driver_symbol_defs.push(
				parse_define_arg(
					report,
					&define_arg)?);
		}

		command.opts.debug_iterations |=
			parsed.opt_present("debug-iters");

		command.opts.optimize_statically_known &=
			!parsed.opt_present("debug-no-optimize-static");

		command.opts.optimize_instruction_matching &=
			!parsed.opt_present("debug-no-optimize-matcher");

		if parsed.opt_present("color")
		{
			command.use_colors = {
				match parsed.opt_str("color").as_ref().map(|s| s.as_ref())
				{
					Some("on") => true,
					Some("off") => false,
					_ =>
					{
						report.error("invalid argument for `--color`");
						return Err(());
					}
				}
			};
		}

		if let Some(t) = parsed.opt_str("t")
		{
			command.opts.max_iterations = {
				match t.parse::<usize>()
				{
					Err(_) | Ok(0) =>
					{
						report.error("invalid argument for `--iters`");
						return Err(());
					}
					Ok(t) => t,
				}
			};
		}


		// Add the input filenames to the main command
		for input_filename in parsed.free.into_iter()
		{
			command.input_filenames.push(input_filename);
		}


		command.output_groups.push(group);
	}


	// Set the default format for each group,
	// if none were specified
	for group in &mut command.output_groups
	{
		if group.format.is_none()
		{
			if group.printout
			{
				group.format = Some(OutputFormat::Annotated {
					base: 16,
					group: 2,
				});
			}
			else
			{
				group.format = Some(OutputFormat::Binary);
			}
		}

		if !group.printout &&
			group.output_filename.is_none() &&
			command.input_filenames.len() >= 1
		{
			group.output_filename = Some(derive_output_filename(
				report,
				&group.format.as_ref().unwrap(),
				&command.input_filenames[0])?);
		}
	}


	Ok(command)
}


fn derive_output_filename(
	report: &mut diagn::Report,
	format: &OutputFormat,
	input_filename: &str)
	-> Result<String, ()>
{
	let extension = {
		match format
		{
			OutputFormat::Binary => "bin",
			OutputFormat::SymbolsMesenMlb => "mlb",
			_ => "txt",
		}
	};

	let mut output_filename = std::path::PathBuf::from(input_filename);
	output_filename.set_extension(extension);

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


pub fn parse_output_format(
	report: &mut diagn::Report,
	format_str: &str)
	-> Result<OutputFormat, ()>
{
	let split = format_str
		.split(',')
		.collect::<Vec<_>>();

	let format_id = split[0];

	let mut params = std::collections::HashMap::<String, String>::new();
	for param in &split[1..]
	{
		let param_split = param
			.split(':')
			.collect::<Vec<_>>();

		let param_id = param_split[0];

		if param_split.len() == 1
		{
			params.insert(param_id.to_string(), "".to_string());
		}
		else if param_split.len() == 2
		{
			params.insert(param_id.to_string(), param_split[1].to_string());
		}
		else
		{
			report.error(
				format!(
					"invalid format argument `{},{}`",
					format_id,
					param));

			return Err(());
		}
	}

	let get_arg_str = &mut |
		params: &mut std::collections::HashMap::<String, String>,
		report: &mut diagn::Report,
		param_id: &str,
		def: &str|
	{
		match params.get(param_id)
		{
			None => Ok(def.to_string()),
			Some(value) =>
			{
				if value.starts_with('\"') &&
					value.ends_with('\"')
				{
					if let Ok(string) = syntax::excerpt_as_string_contents(
						&mut diagn::Report::new(),
						diagn::Span::new_dummy(),
						&value)
					{
						params.remove(param_id);
						return Ok(string);
					}
				}
				
				report.error(
					format!(
						"invalid format argument `{},{}:{}`",
						format_id,
						param_id,
						value));

				Err(())
			}
		}
	};

	let get_arg_usize = &mut |
		params: &mut std::collections::HashMap::<String, String>,
		report: &mut diagn::Report,
		param_id: &str,
		def: usize,
		validate: &mut dyn FnMut(usize)
		-> bool|
	{
		match params.get(param_id)
		{
			None => Ok(def),
			Some(value) =>
			{
				match value.parse::<usize>()
				{
					Ok(v) =>
					{
						if validate(v)
						{
							params.remove(param_id);
							return Ok(v);
						}
					}
					Err(_) => {}
				}

				report.error(
					format!(
						"invalid format argument `{},{}:{}`",
						format_id,
						param_id,
						value));

				Err(())
			}
		}
	};

	let check_nonzero = &mut |value: usize| -> bool
	{
		value > 0
	};

	let check_valid_base = &mut |base: usize| -> bool
	{
		[2, 4, 8, 16, 32, 64, 128].contains(&base)
	};

	let check_2_or_16 = &mut |base: usize| -> bool
	{
		[2, 16].contains(&base)
	};

	let check_8_16_or_32 = &mut |base: usize| -> bool
	{
		[8, 16, 32].contains(&base)
	};

	let format = {
		match format_id
		{
			"binary" => OutputFormat::Binary,

			"annotated" => OutputFormat::Annotated {
				base: get_arg_usize(&mut params, report, "base", 16, check_valid_base)?,
				group: get_arg_usize(&mut params, report, "group", 2, check_nonzero)?,
			},

			"annotatedhex" => OutputFormat::Annotated {
				base: 16,
				group: 2,
			},

			"annotatedbin" => OutputFormat::Annotated {
				base: 2,
				group: 8,
			},

			"binstr" => OutputFormat::BinStr,
			"hexstr" => OutputFormat::HexStr,

			"bindump" => OutputFormat::BinDump,
			"hexdump" => OutputFormat::HexDump,

			"mif" => OutputFormat::Mif,
			"intelhex" => OutputFormat::IntelHex {
				address_unit: get_arg_usize(&mut params, report, "addr_unit", 8, check_8_16_or_32)?,
			},

			"list" => OutputFormat::List(util::FormatListOptions {
				base: get_arg_usize(&mut params, report, "base", 16, check_valid_base)?,
				digits_per_group: get_arg_usize(&mut params, report, "group", 2, check_nonzero)?,
				groups_per_group2: get_arg_usize(&mut params, report, "group2", 16, check_nonzero)?,
				str_before: get_arg_str(&mut params, report, "before", "")?,
				str_after: get_arg_str(&mut params, report, "after", "")?,
				str_between_groups: get_arg_str(&mut params, report, "between", "")?,
				str_between_groups2: get_arg_str(&mut params, report, "between2", "")?,
			}),

			"deccomma" => OutputFormat::DecComma,
			"hexcomma" => OutputFormat::HexComma,
			"decspace" => OutputFormat::DecSpace,
			"hexspace" => OutputFormat::HexSpace,

			"decc" => OutputFormat::DecC,
			"hexc" => OutputFormat::HexC,
			"c" => OutputFormat::HexC,

			"logisim8" => OutputFormat::LogiSim8,
			"logisim16" => OutputFormat::LogiSim16,

			"addrspan" => OutputFormat::AddressSpan,

			"tcgame" => OutputFormat::TCGame {
				base: get_arg_usize(&mut params, report, "base", 16, check_2_or_16)?,
				group: get_arg_usize(&mut params, report, "group", 2, check_nonzero)?,
			},

			"tcgamebin" => OutputFormat::TCGame {
				base: 2,
				group: 8,
			},

			"symbols" => OutputFormat::Symbols,
			"mesen-mlb" => OutputFormat::SymbolsMesenMlb,

			_ =>
			{
				report.error(
					format!(
						"unknown format `{}`",
						format_id));

				return Err(());
			}
		}
	};


	// Error on remaining parameters that were not handled
	for entry in &params
	{
		report.error(
			format!(
				"unknown format argument `{},{}`",
				format_id,
				entry.0));
	}

	if params.len() > 0
	{
		return Err(());
	}


	Ok(format)
}


fn parse_define_arg(
	report: &mut diagn::Report,
	raw_str: &str)
	-> Result<asm::DriverSymbolDef, ()>
{
	let split = raw_str
		.split('=')
		.collect::<Vec<_>>();

	let name = split[0].to_string();

	if split.len() == 1
	{
		return Ok(asm::DriverSymbolDef {
			name,
			value: expr::Value::make_bool(true),
		});
	}

	if split.len() != 2
	{
		report.error(
			format!(
				"invalid define argument `{}`",
				raw_str));

		return Err(());
	}

	let value_str = split[1];

	let value = {
		if value_str == "true"
		{
			expr::Value::make_bool(true)
		}
		else if value_str == "false"
		{
			expr::Value::make_bool(false)
		}
		else
		{
			let has_negative_sign = split[1].chars().next() == Some('-');

			let maybe_value = syntax::excerpt_as_bigint(
				None,
				diagn::Span::new_dummy(),
				if has_negative_sign { split[1].get(1..).unwrap() } else { split[1] });


			use std::ops::Neg;

			match maybe_value
			{
				Ok(value) =>
					expr::Value::make_integer(
						if has_negative_sign { value.neg() } else { value }),

				Err(()) =>
				{
					report.error(
						format!(
							"invalid value for define `{}`",
							name));

					return Err(());
				}
			}
		}
	};


	Ok(asm::DriverSymbolDef {
		name,
		value,
	})
}


pub fn format_output(
	fileserver: &dyn util::FileServer,
	decls: &asm::ItemDecls,
	defs: &asm::ItemDefs,
	output: &util::BitVec,
	format: &OutputFormat)
	-> Vec<u8>
{
	let text = {
		match format
		{
			OutputFormat::Binary =>
				return output.format_binary(),

			OutputFormat::Annotated { base, group } =>
				output.format_annotated(fileserver, *base, *group),

			OutputFormat::TCGame { base, group } =>
				output.format_tcgame(fileserver, *base, *group),

			OutputFormat::BinStr => output.format_binstr(),
			OutputFormat::HexStr => output.format_hexstr(),

			OutputFormat::BinDump => output.format_bindump(),
			OutputFormat::HexDump => output.format_hexdump(),

			OutputFormat::Mif => output.format_mif(),
			OutputFormat::IntelHex { address_unit } =>
				output.format_intelhex(*address_unit),

			OutputFormat::List(opts) => output.format_list(opts),

			OutputFormat::DecComma => output.format_separator(10, ", "),
			OutputFormat::HexComma => output.format_separator(16, ", "),

			OutputFormat::DecSpace => output.format_separator(10, " "),
			OutputFormat::HexSpace => output.format_separator(16, " "),

			OutputFormat::DecC => output.format_c_array(10),
			OutputFormat::HexC => output.format_c_array(16),

			OutputFormat::LogiSim8 => output.format_logisim(8),
			OutputFormat::LogiSim16 => output.format_logisim(16),

			OutputFormat::AddressSpan => output.format_addrspan(fileserver),

			OutputFormat::Symbols => decls.symbols.format_default(decls, defs),
			OutputFormat::SymbolsMesenMlb => decls.symbols.format_mesen_mlb(decls, defs),
		}
	};

	text.bytes().collect()
}


fn print_usage(use_colors: bool)
{
	let usage_str = include_str!("usage_help.md");
	let mut styler = util::StringStyler::new(use_colors);

	for line in usage_str.lines()
	{
		if line.starts_with("# ")
		{
			styler.cyan();
			styler.bold();
			styler.addln(line.get("# ".len()..).unwrap());
			styler.reset();
		}
		else if line.starts_with("## ")
		{
			styler.cyan();
			styler.bold();
			styler.addln(line.get("## ".len()..).unwrap());
			styler.reset();
		}
		else if line.starts_with("`")
		{
			styler.white();
			styler.add("  ");
			styler.addln(&line.replace("`", ""));
			styler.reset();
		}
		else if line.starts_with("* `")
		{
			let cleaned = line
				.get("* `".len()..).unwrap()
				.replace("`", "");

			styler.white();
			styler.add("  ");
			styler.add_styled(
				&cleaned,
				" -- ",
				&mut |s| s.cyan(),
				&mut |s| s.white());
			styler.addln("");
			styler.reset();
		}
		else
		{
			styler.gray();
			styler.add("  ");
			styler.addln(line);
			styler.reset();
		}
	}

	println!("");
	println!("{}", styler.result);
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