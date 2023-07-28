use std::rc::Rc;

use crate::{asm::Disassembler, diagn::Span, *};
use getopts;

enum InputFormat {
    Binary,
    BinStr,
    BinLine,
    HexStr,
    HexLine,
}

enum OutputFormat {
    Binary,
    AnnotatedHex,
    AnnotatedBin,
    BinStr,
    BinLine,
    HexStr,
    HexLine,
    BinDump,
    HexDump,
    Coe,
    Mif,
    MifBin,
    IntelHex,
    DecComma,
    HexComma,
    DecC,
    HexC,
    BinVHDL,
    HexVHDL,
    LogiSim8,
    LogiSim16,
    AddressSpan,
}

enum SymbolFormat {
    Default,
    MesenMlb,
}

pub fn drive(args: &Vec<String>, fileserver: &mut dyn util::FileServer) -> Result<(), ()> {
    let opts = make_opts();

    let report = diagn::RcReport::new();

    let result = drive_inner(report.clone(), &opts, args, fileserver);

    if report.has_messages() {
        println!("");
    }

    util::enable_windows_ansi_support();
    report.print_all(&mut std::io::stderr(), fileserver);

    if let Err(show_usage) = result {
        if show_usage {
            print_version();
            print_usage(&opts);
        }
    }

    result.map_err(|_| ())
}

fn drive_inner(
    report: diagn::RcReport,
    opts: &getopts::Options,
    args: &Vec<String>,
    fileserver: &mut dyn util::FileServer,
) -> Result<(), bool> {
    let matches = parse_opts(report.clone(), opts, args).map_err(|_| true)?;

    if matches.opt_present("h") {
        print_usage(&opts);
        return Ok(());
    }

    if matches.opt_present("v") {
        print_version();
        return Ok(());
    }

    let quiet = matches.opt_present("q");
    let out_stdout = matches.opt_present("p");

    let max_iterations = match matches.opt_str("t") {
        None => 10,
        Some(t) => match t.parse::<usize>() {
            Ok(t) => t,
            Err(_) => {
                report.error("invalid number of iterations");
                return Err(true);
            }
        },
    };

    if matches.free.len() < 1 {
        report.error("no input files");
        return Err(true);
    }

    let main_asm_file = matches.free[0].clone();

    let output_symbol_requested = matches.opt_present("s");
    let output_requested = matches.opt_present("o");

    let output_symbol_file = matches.opt_str("s");
    let output_file = match matches.opt_str("o") {
        Some(f) => Some(f),
        None => {
            if output_symbol_requested || output_symbol_file.is_some() {
                None
            } else {
                match get_default_output_filename(
                    report.clone(),
                    &match matches.opt_str("d") {
                        Some(n) => n,
                        None => main_asm_file,
                    },
                    matches.opt_present("d"),
                ) {
                    Ok(f) => Some(f),
                    Err(_) => None,
                }
            }
        }
    };

    if !quiet {
        print_version();
    }

    let mut assembler = asm::Assembler::new();
    for filename in matches.free.clone() {
        if !quiet {
            println!("assembling `{}`...", filename);
        }

        assembler.register_file(filename);
    }

    let output = assembler
        .assemble(report.clone(), fileserver, max_iterations)
        .map_err(|_| false)?;

    match matches.opt_str("d") {
        None => {}
        Some(f) => {
            let in_format = match matches.opt_str("f").as_ref().map(|s| s.as_ref()) {
                Some("binary") => InputFormat::Binary,
                Some("binstr") => InputFormat::BinStr,
                Some("binline") => InputFormat::BinLine,
                Some("hexstr") => InputFormat::HexStr,
                Some("hexline") => InputFormat::HexLine,

                None => InputFormat::Binary,

                Some(_) => {
                    report.error("invalid input format");
                    return Err(true);
                }
            };

            let out = Disassembler::new(
                output.state.rulesets,
                output.state.active_rulesets,
                match in_format {
                    InputFormat::Binary => util::BitVec::parse_binary(
                        fileserver
                            .get_bytes(report.clone(), &f, None)
                            .map_err(|_| false)?,
                        output.state.cur_wordsize,
                    ),
                    InputFormat::BinStr => util::BitVec::parse_binstr(
                        String::from_iter(
                            fileserver
                                .get_chars(report.clone(), &f, None)
                                .map_err(|_| false)?,
                        ),
                        output.state.cur_wordsize,
                    ),
                    InputFormat::BinLine => util::BitVec::parse_binline(
                        String::from_iter(
                            fileserver
                                .get_chars(report.clone(), &f, None)
                                .map_err(|_| false)?,
                        ),
                        output.state.cur_wordsize,
                    ),
                    InputFormat::HexStr => util::BitVec::parse_hexstr(
                        String::from_iter(
                            fileserver
                                .get_chars(report.clone(), &f, None)
                                .map_err(|_| false)?,
                        ),
                        output.state.cur_wordsize,
                    ),
                    InputFormat::HexLine => util::BitVec::parse_hexline(
                        String::from_iter(
                            fileserver
                                .get_chars(report.clone(), &f, None)
                                .map_err(|_| false)?,
                        ),
                        output.state.cur_wordsize,
                    ),
                },
                match matches.opt_str("n").as_ref().map(|s| s.as_ref()) {
                    Some("hex") => asm::NumberFormat::Hex,
                    Some("bin") => asm::NumberFormat::Bin,
                    Some("dec") => asm::NumberFormat::Dec,

                    None => asm::NumberFormat::Hex,

                    Some(_) => {
                        report.error("invalid number format");
                        return Err(true);
                    }
                },
            )
            .disassemble(report.clone())
            .map_err(|_| false)?;

            if out_stdout {
                if output_requested || output_file.is_some() {
                    println!("{}", out.assembly);
                }
            } else {
                let mut any_files_written = false;

                if let Some(ref output_file) = output_file {
                    let mut o = "".to_string();

                    for f in matches.free.clone() {
                        o += &format!(
                            "#include \"{}\"\n",
                            util::filename_navigate(
                                report.clone(),
                                output_file,
                                &f,
                                &Span::new(Rc::new("".to_string()), 0, 0)
                            )
                            .unwrap()
                            .replace("\\", "/")
                        );
                    }

                    o += "\n";

                    o += &out.assembly;

                    println!("writing `{}`...", &output_file);
                    fileserver
                        .write_bytes(report.clone(), &output_file, &o.into_bytes(), None)
                        .map_err(|_| false)?;
                    any_files_written = true;
                }

                if !any_files_written {
                    println!("no files written");
                }
            }

            return Ok(());
        }
    }

    let out_format = match matches.opt_str("f").as_ref().map(|s| s.as_ref()) {
        Some("annotated") => OutputFormat::AnnotatedHex,
        Some("annotatedhex") => OutputFormat::AnnotatedHex,
        Some("annotatedbin") => OutputFormat::AnnotatedBin,

        Some("binstr") => OutputFormat::BinStr,
        Some("binline") => OutputFormat::BinLine,
        Some("bindump") => OutputFormat::BinDump,
        Some("hexstr") => OutputFormat::HexStr,
        Some("hexline") => OutputFormat::HexLine,
        Some("hexdump") => OutputFormat::HexDump,
        Some("binary") => OutputFormat::Binary,
        Some("coe") => OutputFormat::Coe,
        Some("mif") => OutputFormat::Mif,
        Some("mifbin") => OutputFormat::MifBin,
        Some("intelhex") => OutputFormat::IntelHex,
        Some("deccomma") => OutputFormat::DecComma,
        Some("hexcomma") => OutputFormat::HexComma,
        Some("decc") => OutputFormat::DecC,
        Some("hexc") => OutputFormat::HexC,
        Some("c") => OutputFormat::HexC,
        Some("binvhdl") => OutputFormat::BinVHDL,
        Some("hexvhdl") => OutputFormat::HexVHDL,
        Some("vhdl") => OutputFormat::HexVHDL,
        Some("logisim8") => OutputFormat::LogiSim8,
        Some("logisim16") => OutputFormat::LogiSim16,
        Some("addrspan") => OutputFormat::AddressSpan,

        None => {
            if out_stdout {
                OutputFormat::AnnotatedHex
            } else {
                OutputFormat::Binary
            }
        }

        Some(_) => {
            report.error("invalid output format");
            return Err(true);
        }
    };

    let symbol_format = match matches
        .opt_str("symbol-format")
        .as_ref()
        .map(|s| s.as_ref())
    {
        None | Some("default") => SymbolFormat::Default,
        Some("mesen-mlb") => SymbolFormat::MesenMlb,
        Some(_) => {
            report.error("invalid symbol format");
            return Err(true);
        }
    };

    let binary = output.binary;

    let output_symbol_data = if output_symbol_file.is_none() {
        None
    } else {
        Some(match symbol_format {
            SymbolFormat::Default => output.state.symbols.format_default(),
            SymbolFormat::MesenMlb => output.state.symbols.format_mesen_mlb(&output.state),
        })
    };

    let output_data: Vec<u8> = match out_format {
        OutputFormat::Binary => binary.format_binary(),

        OutputFormat::BinStr => binary.format_binstr().bytes().collect(),
        OutputFormat::BinLine => binary
            .format_binline(output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::HexStr => binary.format_hexstr().bytes().collect(),
        OutputFormat::HexLine => binary
            .format_hexline(output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::BinDump => binary.format_bindump().bytes().collect(),
        OutputFormat::HexDump => binary.format_hexdump().bytes().collect(),
        OutputFormat::Coe => binary
            .format_coe(output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::Mif => binary
            .format_mif(4, output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::MifBin => binary
            .format_mif(1, output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::IntelHex => binary.format_intelhex().bytes().collect(),
        OutputFormat::DecComma => binary.format_comma(10).bytes().collect(),
        OutputFormat::HexComma => binary.format_comma(16).bytes().collect(),
        OutputFormat::DecC => binary.format_c_array(10).bytes().collect(),
        OutputFormat::HexC => binary.format_c_array(16).bytes().collect(),
        OutputFormat::BinVHDL => binary
            .format_vhdl_b_array(output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::HexVHDL => binary
            .format_vhdl_h_array(output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::LogiSim8 => binary.format_logisim(8).bytes().collect(),
        OutputFormat::LogiSim16 => binary.format_logisim(16).bytes().collect(),

        OutputFormat::AnnotatedHex => binary
            .format_annotated_hex(fileserver, output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::AnnotatedBin => binary
            .format_annotated_bin(fileserver, output.state.cur_wordsize)
            .bytes()
            .collect(),
        OutputFormat::AddressSpan => binary.format_addrspan(fileserver).bytes().collect(),
    };

    if out_stdout {
        if !quiet {
            println!(
                "success after {} iteration{}",
                output.iterations,
                if output.iterations == 1 { "" } else { "s" }
            );
            println!("");
        }

        if output_requested || output_file.is_some() {
            println!("{}", String::from_utf8_lossy(&output_data));
        }

        if output_symbol_requested || output_symbol_file.is_some() {
            if let Some(output_symbol_data) = output_symbol_data {
                println!("{}", &output_symbol_data);
            }
        }
    } else {
        let mut any_files_written = false;

        if let Some(ref output_file) = output_file {
            println!("writing `{}`...", &output_file);
            fileserver
                .write_bytes(report.clone(), &output_file, &output_data, None)
                .map_err(|_| false)?;
            any_files_written = true;
        }

        if let Some(output_symbol_data) = output_symbol_data {
            if let Some(ref output_symbol_file) = output_symbol_file {
                println!("writing `{}`...", &output_symbol_file);
                fileserver
                    .write_bytes(
                        report.clone(),
                        &output_symbol_file,
                        &output_symbol_data.bytes().collect::<Vec<u8>>(),
                        None,
                    )
                    .map_err(|_| false)?;
                any_files_written = true;
            }
        }

        if !any_files_written {
            println!("no files written");
        }

        if !quiet {
            println!(
                "success after {} iteration{}",
                output.iterations,
                if output.iterations == 1 { "" } else { "s" }
            );
        }
    }

    Ok(())
}

fn make_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.opt(
        "d",
        "disassemble",
        "Disassemble",
        "FILE",
        getopts::HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "n",
        "number-format",
        "The format of numbers in the disassembly output. Possible formats: hex, bin, dec",
        "FILE",
        getopts::HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optopt("f", "format", "The format of the output (input for disassembly, compatible modes have a star) file. Possible formats: binary*, annotated, annotatedbin, binstr*, binline*, hexstr*, hexline*, bindump, hexdump, mif, intelhex, deccomma, hexcomma, decc, hexc, binvhdl, hexvhdl, logisim8, logisim16, addrspan", "FORMAT");
    opts.opt(
        "o",
        "output",
        "The name of the output file.",
        "FILE",
        getopts::HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optopt(
        "",
        "symbol-format",
        "The format of the symbol file. Possible formats: default, mesen-mlb",
        "SYMBOL-FORMAT",
    );
    opts.opt(
        "s",
        "symbol",
        "The name of the output symbol file.",
        "FILE",
        getopts::HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "t",
        "iter",
        "The max number of passes the assembler will attempt (default: 10).",
        "NUM",
        getopts::HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optflag(
        "p",
        "print",
        "Print output to stdout instead of writing to a file.",
    );
    opts.optflag("q", "quiet", "Suppress progress reports.");
    opts.optflag("v", "version", "Display version information.");
    opts.optflag("h", "help", "Display this information.");

    opts
}

fn parse_opts(
    report: diagn::RcReport,
    opts: &getopts::Options,
    args: &Vec<String>,
) -> Result<getopts::Matches, ()> {
    match opts.parse(&args[1..]) {
        Ok(m) => Ok(m),
        Err(f) => Err(report.error(format!("{}", f))),
    }
}

fn print_usage(opts: &getopts::Options) {
    println!("");
    println!(
        "{}",
        opts.usage(&format!(
            "Usage: {} [options] <asm-file-1> ... <asm-file-N>",
            env!("CARGO_PKG_NAME")
        ))
    );
}

fn print_version() {
    let mut date = format!("{}, ", env!("VERGEN_GIT_COMMIT_DATE"));
    if date == "UNKNOWN, " {
        date = "".to_string();
    }

    println!(
        "{} {} ({}{})",
        env!("CARGO_PKG_NAME"),
        env!("VERGEN_GIT_DESCRIBE"),
        date,
        env!("VERGEN_CARGO_TARGET_TRIPLE")
    );
}

fn get_default_output_filename(
    report: diagn::RcReport,
    input_filename: &str,
    disassembly: bool,
) -> Result<String, ()> {
    use std::path::PathBuf;

    let mut output_filename = PathBuf::from(input_filename);
    output_filename.set_extension(if disassembly { "asm" } else { "bin" });

    let output_filename = output_filename
        .to_string_lossy()
        .into_owned()
        .replace("\\", "/");

    if output_filename == input_filename {
        return Err(report.error("cannot derive safe output filename"));
    }

    Ok(output_filename)
}
