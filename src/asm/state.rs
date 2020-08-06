use crate::*;


pub struct State
{
	pub patterns: Vec<usize>,//Vec<Pattern>,
	pub pattern_invoks: Vec<usize>,//Vec<PatternInvokation>,
}


impl State
{
	pub fn new() -> State
	{
		let state = State
		{
			patterns: Vec::new(),
			pattern_invoks: Vec::new(),
		};
		
		state
	}
	
	
	pub fn process_file<S: Into<String>>(
        &mut self,
        report: diagn::RcReport,
        fileserver: &dyn util::FileServer,
        filename: S)
        -> Result<(), ()>
	{
        asm::parse_file(report.clone(), self, fileserver, filename);
		
		match report.has_errors()
		{
			true => Err(()),
			false => Ok(())
		}
    }
}