extern crate customasm;


use std::fs::File;
use std::io::Read;


fn main()
{
	let mut src = String::new();
	File::open("def.txt").unwrap().read_to_string(&mut src).unwrap();
	
	match customasm::Configuration::from_src(&mut src.chars())
	{
		Err(err) => println!("error:{}:{}: {}", err.line_num, err.column_num, err.msg),
		_ => println!("success")
	}
}