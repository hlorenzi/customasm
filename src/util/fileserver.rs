use diagn::Message;
use std::collections::HashMap;


pub trait FileServer
{
	fn get_bytes(&self, filename: &str) -> Result<Vec<u8>, Message>;
	
	
	fn get_chars(&self, filename: &str) -> Result<Vec<char>, Message>
	{
		let bytes = self.get_bytes(filename)?;
		
		Ok(String::from_utf8_lossy(&bytes).chars().collect())
	}
}


pub struct FileServerMock
{
	files: HashMap<String, Vec<u8>>
}


impl FileServerMock
{
	pub fn new() -> FileServerMock
	{
		FileServerMock
		{
			files: HashMap::new()
		}
	}
	
	
	pub fn add<S, T>(&mut self, filename: S, contents: T)
	where S: Into<String>, T: Into<Vec<u8>>
	{
		self.files.insert(filename.into(), contents.into());
	}
}


impl FileServer for FileServerMock
{
	fn get_bytes(&self, filename: &str) -> Result<Vec<u8>, Message>
	{
		match self.files.get(filename)
		{
			None => Err(Message::error(format!("file not found: `{}`", filename))),
			Some(bytes) => Ok(bytes.clone())
		}
	}
}