use diagn::RcReport;
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


pub fn read_instrset<S>(report: RcReport, fileserver: &FileServer, filename: S) -> Result<InstrSet, ()>
where S: Into<String>
{
	let filename_owned = filename.into();
	let chars = fileserver.get_chars(report.clone(), &filename_owned, None)?;
	let tokens = tokenize(report.clone(), filename_owned, &chars)?;
	let instrset = InstrSetParser::new(report.clone(), tokens).parse()?;
	
	match report.has_errors()
	{
		true => Err(()),
		false => Ok(instrset)
	}
}