use diagn::Reporter;
use syntax::Token;
use super::InstrSetParser;
use instrset::Rule;


#[derive(Debug)]
pub struct InstrSet
{
	pub align: usize,
	pub rules: Vec<Rule>
}


impl InstrSet
{
	pub fn from_tokens(reporter: &mut Reporter, tokens: &[Token]) -> Option<InstrSet>
	{
		let mut instrset = InstrSet
		{
			align: 8,
			rules: Vec::new()
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