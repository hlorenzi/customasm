use util::error::Error;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};


pub trait FileHandler
{
	fn read_bytes(&self, filename: &Path) -> Result<Vec<u8>, Error>;
	
	
	fn read_str(&self, filename: &Path) -> Result<String, Error>
	{
		let bytes = try!(self.read_bytes(filename));
		Ok(String::from_utf8_lossy(&bytes).into_owned())
	}
	
	
	fn read_chars(&self, filename: &Path) -> Result<Vec<char>, Error>
	{
		let bytes = try!(self.read_bytes(filename));
		Ok(String::from_utf8_lossy(&bytes).into_owned().chars().collect())
	}
}


pub struct CustomFileHandler
{
	files: HashMap<PathBuf, Vec<u8>>
}


impl CustomFileHandler
{
	pub fn new() -> CustomFileHandler
	{
		CustomFileHandler
		{
			files: HashMap::new()
		}
	}
	
	
	pub fn add<P, C>(&mut self, filename: P, contents: C)
	where P: Into<PathBuf>, C: Into<Vec<u8>>
	{
		self.files.insert(filename.into(), contents.into());
	}
}


impl FileHandler for CustomFileHandler
{
	fn read_bytes(&self, filename: &Path) -> Result<Vec<u8>, Error>
	{
		if !self.files.contains_key(filename)
			{ Err(Error::new(format!("file does not exist: `{}`", filename.to_string_lossy()))) }
		else
			{ Ok(self.files[filename].clone()) }
	}
}


pub struct RealFileHandler;


impl RealFileHandler
{
	pub fn new() -> RealFileHandler
	{
		RealFileHandler
	}
}


impl FileHandler for RealFileHandler
{
	fn read_bytes(&self, filename: &Path) -> Result<Vec<u8>, Error>
	{
		if !filename.exists()
		{
			return 
				Err(Error::new(format!("file does not exist: `{}`", filename.to_string_lossy())));
		}
		
		let mut file = match File::open(filename)
		{
			Ok(file) => file,
			Err(err) => return
				Err(Error::new(format!("could not open file `{}`: {}", filename.to_string_lossy(), err)))
		};

		let mut vec = Vec::new();
		match file.read_to_end(&mut vec)
		{
			Ok(..) => Ok(vec),
			Err(err) =>
				Err(Error::new(format!("could not read file `{}`: {}", filename.to_string_lossy(), err)))
		}
	}
}