use customasm::*;


fn main()
{
	let args: Vec<String> = std::env::args().collect();
	
	let mut fileserver = util::FileServerReal::new();
	
	if let Err(()) = driver::drive(&args, &mut fileserver)
	{
		std::process::exit(1);
	}
}