pub struct BinaryBlock
{
	pub bank_name: String,
	pub bits: Vec<bool>
}


impl BinaryBlock
{
	pub fn new<S>(bank_name: S) -> BinaryBlock
	where S: Into<String>
	{
		BinaryBlock
		{
			bank_name: bank_name.into(),
			bits: Vec::new()
		}
	}
	
	
	pub fn append(&mut self, bit: bool)
	{
		let index = self.len();
		self.write(index, bit);
	}
	
	
	pub fn truncate(&mut self, new_len: usize)
	{
		while self.bits.len() > new_len
			{ self.bits.pop(); }
	}
	
	
	pub fn write(&mut self, index: usize, bit: bool)
	{
		while self.bits.len() <= index
			{ self.bits.push(false); }
			
		self.bits[index] = bit;
	}
	
	
	pub fn read(&self, index: usize) -> bool
	{
		if index >= self.bits.len()
			{ false }
		else
			{ self.bits[index] }
	}
	
	
	pub fn len(&self) -> usize
	{
		self.bits.len()
	}
}