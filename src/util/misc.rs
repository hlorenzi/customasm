use util::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;


pub fn error_exit(err: Error) -> !
{
	err.print();
	exit(1);
}


pub fn read_file(path: &Path) -> Vec<char>
{
	let mut file = match File::open(path)
	{
		Ok(file) => file,
		Err(err) => error_exit(Error::new_with_file(path.to_string_lossy().into_owned(), format!("{}", err)))
	};

	let mut s = String::new();
	match file.read_to_string(&mut s)
	{
		Ok(..) => s.chars().collect::<Vec<char>>(),
		Err(err) => error_exit(Error::new_with_file(path.to_string_lossy().into_owned(), format!("{}", err)))
	}
}


pub fn read_file_bytes(path: &Path) -> Vec<u8>
{
	let mut file = match File::open(path)
	{
		Ok(file) => file,
		Err(err) => error_exit(Error::new_with_file(path.to_string_lossy().into_owned(), format!("{}", err)))
	};

	let mut vec = Vec::new();
	match file.read_to_end(&mut vec)
	{
		Ok(..) => vec,
		Err(err) => error_exit(Error::new_with_file(path.to_string_lossy().into_owned(), format!("{}", err)))
	}
}