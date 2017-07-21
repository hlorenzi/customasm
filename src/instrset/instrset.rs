use diagn::Reporter;
use syntax::Token;
use super::InstrSetParser;


pub struct InstrSet
{
	pub align: usize
}


impl InstrSet
{
	pub fn from_tokens(reporter: &mut Reporter, tokens: &[Token]) -> Option<InstrSet>
	{
		let mut instrset = InstrSet
		{
			align: 8
		};
		
		match InstrSetParser::new(reporter, tokens, &mut instrset).parse()
		{
			Ok(()) => Some(instrset),
			Err(msg) => 
			{
				reporter.message(msg);
				None
			}
		}
	}
}