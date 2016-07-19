extern crate customassembly;


use std::fs::File;
use std::io::Read;


fn main()
{
	let mut src = String::new();
	File::open("def.txt").unwrap().read_to_string(&mut src).unwrap();
	
	match customassembly::Configuration::parse(&mut src.chars())
	{
		Err(err) => println!("error:{}:{}: {}", err.line_num, err.column_num, err.msg),
		_ => println!("success")
	}
}