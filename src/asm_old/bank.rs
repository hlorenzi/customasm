use crate::asm::BinaryBlock;


pub struct Bank
{
	pub bank_name: String,
	pub bits: BinaryBlock
}


impl Bank
{
	pub fn new<S>(bank_name: S) -> Bank
	where S: Into<String>
	{
		Bank
		{
			bank_name: bank_name.into(),
			bits: BinaryBlock::new()
		}
	}
}