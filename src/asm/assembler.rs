use diagn::Reporter;
use instrset::InstrSet;
use util::FileServer;
use asm::{AssemblerParser, RulePatternMatcher};



pub struct AssemblerState<'a>
{
	pub reporter: &'a mut Reporter,
	pub fileserver: &'a FileServer,
	pub instrset: &'a InstrSet,
	pub pattern_matcher: RulePatternMatcher
}


pub fn assemble<S>(reporter: &mut Reporter, instrset: &InstrSet, fileserver: &FileServer, filename: S) -> Option<()>
where S: Into<String>
{
	let pattern_matcher = RulePatternMatcher::new(&instrset.rules);
	println!("{:#?}", pattern_matcher);
	
	let mut state = AssemblerState
	{
		reporter: reporter,
		fileserver: fileserver,
		instrset: instrset,
		pattern_matcher: pattern_matcher
	};
	
	let output = match AssemblerParser::parse_file(&mut state, filename)
	{
		Ok(output) => output,
		Err(msg) =>
		{
			state.reporter.message(msg);
			return None;
		}
	};
	
	if state.reporter.has_errors()
		{ return None; }
	
	Some(output)
}