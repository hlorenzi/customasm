use diagn::Report;
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


pub fn read_instrset<S>(report: &mut Report, fileserver: &FileServer, filename: S) -> Result<InstrSet, ()>
where S: Into<String>
{
	let filename_owned = filename.into();
	let chars = fileserver.get_chars(report, &filename_owned, None)?;
	let tokens = tokenize(report, filename_owned, &chars)?;
	let instrset = InstrSetParser::new(report, &tokens).parse()?;
	
	match report.has_errors()
	{
		true => Err(()),
		false => Ok(instrset)
	}
}