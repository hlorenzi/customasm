extern crate customasm;


use std::fs::File;
use std::io::Read;


fn main()
{
	let mut cfg_src = String::new();
	File::open("def.txt").unwrap().read_to_string(&mut cfg_src).unwrap();
	let cfg_chars = cfg_src.chars().collect::<Vec<_>>();
	
	//println!("{:#?}", customasm::tokenize(&src.chars().collect::<Vec<_>>()));
	
	println!("parsing configuration...");
	let cfg = match customasm::Configuration::from_src(&cfg_chars)
	{
		Ok(cfg) => cfg,
		Err(err) =>
		{
			let (line, column) = err.span.get_line_column(&cfg_chars);
			println!("");
			println!("error:{}:{}: {}", line, column, err.msg);
			return;
		}
	};
	
	
	let mut asm_src = String::new();
	File::open("input.asm").unwrap().read_to_string(&mut asm_src).unwrap();
	let asm_chars = asm_src.chars().collect::<Vec<_>>();
	
	println!("assembling...");
	let _output = match customasm::translate(&cfg, &asm_chars)
	{
		Ok(output) => output,
		Err(err) =>
		{
			let (line, column) = err.span.get_line_column(&asm_chars);
			println!("");
			println!("error:{}:{}: {}", line, column, err.msg);
			return;
		}
	};
	
	println!("success");
}