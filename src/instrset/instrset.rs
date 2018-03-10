use instrset::Rule;
use asm::RulePatternMatcher;


#[derive(Debug)]
pub struct InstrSet
{
	pub align: usize,
	pub rules: Vec<Rule>,
	pub pattern_matcher: RulePatternMatcher
}