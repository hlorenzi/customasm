use std::cmp::Reverse;
use core::cmp::Ordering;
use crate::syntax::TokenKind;
use crate::diagn::RcReport;
use crate::syntax::Parser;
use crate::expr::{Expression, ExpressionValue};
use crate::asm::cpudef::{Rule, RuleParameterType, RulePatternPart, CustomTokenDef};
use std::{rc::Rc, collections::HashMap, cmp::max};


/// there is no "sequence" pattern, since how that sequence becomes an output expression varies
#[derive(Debug, Eq, PartialEq)]
enum PatternComponent {
	End,
	Exact(TokenKind, Option<String>),
	Expression
}

pub struct Match
{
	pub rule_indices: Vec<usize>,
	pub exprs: Vec<Expression>
}

impl PatternComponent {

	fn part(part: &RulePatternPart, rule: &Rule, custom_token_defs: &Vec<CustomTokenDef>) -> PatternComponent {
		match part {
			RulePatternPart::Exact(kind, exceprt) => {
				PatternComponent::Exact(*kind, exceprt.as_ref().map(|it| it.to_ascii_lowercase().into()))
			},
			RulePatternPart::Parameter(param_index) => {
				match rule.params[*param_index].typ {
					RuleParameterType::Expression |
					RuleParameterType::Unsigned(_) |
					RuleParameterType::Signed(_) |
					RuleParameterType::Integer(_) => PatternComponent::Expression,

					RuleParameterType::CustomTokenDef(def_index) => {
						let token_def: &CustomTokenDef = &custom_token_defs[def_index];

						PatternComponent::Expression
					},
				}
			}
		}
	}

	/// Ok(_) means there was a match, Err(()) means there wasn't
	fn get_match(&self, parser: &mut Parser) -> Result<Option<Expression>, ()> {
		match self {
			PatternComponent::End => {
				if parser.next_is_linebreak() {
					return Ok(None);
				}
			},
			PatternComponent::Exact(kind, ref exceprt) => {
				let tk = parser.advance();
				if tk.kind == *kind && tk.excerpt == *exceprt {
					return Ok(None);
				}
			},
			PatternComponent::Expression => {
				if let Ok(expr) = Expression::parse(parser) {
					return Ok(Some(expr));
				}
			}
		}

		Err(())
	}
}

#[derive(Debug)]
struct InstrPattern {
	rule_indices: Vec<usize>,
	components: Vec<PatternComponent>,
}

#[derive(PartialEq, Eq)]
struct InstrSpecificity {
	param_indices: Vec<usize>
}

impl InstrSpecificity {
	fn create(pattern: &InstrPattern) -> InstrSpecificity {
		let mut param_indices: Vec<usize> = vec![];
		for i in 0 .. pattern.components.len() {
			match pattern.components[i] {
				PatternComponent::End | PatternComponent::Exact(_, _) => {},
				_ => param_indices.push(i)
			}
		}
		InstrSpecificity {
			param_indices
		}
	}
}

impl Ord for InstrSpecificity {
    fn cmp(&self, other: &Self) -> Ordering {
		let self_count = self.param_indices.len();
		let other_count = other.param_indices.len();
		for i in 0 .. max(self_count, other_count) {
			if i < self_count && i < other_count {
				match self.param_indices[i].cmp(&other.param_indices[i]) {
					Ordering::Less => return Ordering::Less,
					Ordering::Equal => {},
					Ordering::Greater => return Ordering::Greater,
				}
			}
			if i < self_count && i >= other_count {
				return Ordering::Less;
			}
			if i < other_count && i >= self_count {
				return Ordering::Greater;
			}
		}
		Ordering::Equal
    }
}

impl PartialOrd for InstrSpecificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}



// matcher

#[derive(Debug)]
pub struct RulePatternMatcher
{
	patterns: Vec<InstrPattern>
}

impl RulePatternMatcher
{
	pub fn new(report: RcReport, rules: &[Rule], custom_token_defs: &Vec<CustomTokenDef>) -> Result<RulePatternMatcher, ()>
	{
		let mut patterns: Vec<InstrPattern> = vec![];

		for i in 0..rules.len() {
			let rule = &rules[i];
			let components = rule.pattern_parts.iter()
				.map(|part| PatternComponent::part(part, rule, custom_token_defs))
				.chain(vec![PatternComponent::End]).collect::<Vec<PatternComponent>>();
			if let Some(pattern) = patterns.iter_mut().find_map(|pattern| if pattern.components == components { Some(pattern) } else { None }) {
				pattern.rule_indices.push(i);
			} else {
				patterns.push(InstrPattern {
					rule_indices: vec![i],
					components,
				});
			}

		}

		patterns.sort_by_cached_key(|it| Reverse(InstrSpecificity::create(it)));
		
		Ok(RulePatternMatcher
		{
			patterns
		})
	}
	
	pub fn parse_match(&self, parser: &mut Parser) -> Option<Match>
	{
		let starting_state = parser.save();
		'patterns: for pattern in self.patterns.iter() {
			let mut expressions: Vec<Expression> = Vec::new();
			for component in &pattern.components {
				match component.get_match(parser) {
					Err(_) => {
						parser.restore(starting_state);
						continue 'patterns;
					},
					Ok(Some(expr)) => expressions.push(expr),
					Ok(_) => {}
				}
			}
			return Some(Match {
				rule_indices: pattern.rule_indices.clone(),
				exprs: expressions,
			});
		}

		None
	}
}