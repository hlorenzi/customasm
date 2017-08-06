extern crate customasm;


fn main()
{
	let args: Vec<String> = std::env::args().collect();
	
	let mut fileserver = customasm::FileServerReal::new();
	
	if let Err(()) = customasm::drive(&args, &mut fileserver)
		{ std::process::exit(1); }
}