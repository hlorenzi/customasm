use diagn::Reporter;
use syntax::tokenize;
use util::FileServer;
use super::InstrSetParser;
use instrset::Rule;


#[derive(Debug)]
pub struct InstrSet
{
	pub align: usize,
	pub rules: Vec<Rule>
}


pub fn read_instrset<S>(reporter: &mut Reporter, fileserver: &FileServer, filename: S) -> Option<InstrSet>
where S: Into<String>
{
	let filename_owned = filename.into();
	let chars = match fileserver.get_chars(&filename_owned)
	{
		Ok(chars) => chars,
		Err(msg) =>
		{
			reporter.message(msg);
			return None;
		}
	};
	
	let tokens = tokenize(reporter, filename_owned, &chars);
	
	let instrset = match InstrSetParser::new(reporter, &tokens).parse()
	{
		Ok(instrset) => instrset,
		Err(msg) => 
		{
			reporter.message(msg);
			return None;
		}
	};
	
	if reporter.has_errors()
		{ return None; }
	
	Some(instrset)
}