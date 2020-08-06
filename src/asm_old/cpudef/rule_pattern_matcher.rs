use crate::diagn::RcReport;
use crate::syntax::Parser;
use crate::expr::{Expression, ExpressionValue};
use crate::asm::cpudef::{Rule, RuleParameterType, RulePatternPart, CustomTokenDef};
use std::collections::HashMap;


#[derive(Debug)]
pub struct RulePatternMatcher
{
	root_step: MatchStep
}


#[derive(Debug)]
struct MatchStep
{
	rule_indices: Vec<usize>,
	children_exact: HashMap<char, Vec<(Option<ExpressionValue>, Box<MatchStep>)>>,
	children_param: Option<Box<MatchStep>>
}


#[derive(Debug)]
pub struct Match
{
	pub rule_indices: Vec<usize>,
	pub exprs: Vec<Expression>
}


impl RulePatternMatcher
{
	pub fn new(report: RcReport, rules: &[Rule], custom_token_defs: &Vec<CustomTokenDef>) -> Result<RulePatternMatcher, ()>
	{
		let mut root_step = MatchStep::new();
		
		for i in 0..rules.len()
			{ RulePatternMatcher::add_tree_step(report.clone(), &mut root_step, &rules[i], &rules[i].pattern_parts, i, custom_token_defs)?; }
		
		Ok(RulePatternMatcher
		{
			root_step: root_step
		})
	}
	

	fn add_tree_step(report: RcReport, step: &mut MatchStep, rule: &Rule, next_parts: &[RulePatternPart], rule_index: usize, custom_token_defs: &Vec<CustomTokenDef>) -> Result<(), ()>
	{
		if next_parts.len() == 0
		{
			step.rule_indices.push(rule_index);
			step.rule_indices.sort();
			return Ok(());
		}
		
		match next_parts[0]
		{
			RulePatternPart::Exact(c) =>
			{
				let next_step = Self::make_step_exact(step, c.to_ascii_lowercase(), None);
				RulePatternMatcher::add_tree_step(report.clone(), next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
			}
			
			RulePatternPart::Parameter(param_index) =>
			{
				if let RuleParameterType::CustomTokenDef(tokendef_index) = rule.params[param_index].typ
				{
					let custom_token_def = &custom_token_defs[tokendef_index];
					
					for (excerpt, value) in &custom_token_def.excerpt_to_value_map
					{
						let chars = excerpt.chars().collect::<Vec<char>>();
						let mut next_step = Self::make_step_exact(
							step,
							chars[0].to_ascii_lowercase(),
							Some(value.clone()));

						if chars.len() > 1
						{
							for c in &chars[1..chars.len()]
							{
								next_step = Self::make_step_exact(
									next_step,
									c.to_ascii_lowercase(),
									None);
							}
						}

						RulePatternMatcher::add_tree_step(report.clone(), next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
					}
				}
				else
				{
					let next_step = Self::make_step_param(step);
					RulePatternMatcher::add_tree_step(report.clone(), next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
				}
			}
		}
		
		return Ok(());
	}


	fn make_step_exact(step: &mut MatchStep, c: char, value: Option<ExpressionValue>) -> &mut MatchStep
	{
		if !step.children_exact.contains_key(&c)
		{
			step.children_exact.insert(c, vec![(value, Box::new(MatchStep::new()))]);
			return &mut step.children_exact.get_mut(&c).unwrap().last_mut().unwrap().1;
		}

		let next_steps = step.children_exact.get_mut(&c).unwrap();

		if next_steps.iter_mut().find(|s| s.0 == value).is_none()
		{
			next_steps.push((value, Box::new(MatchStep::new())));
			return &mut next_steps.last_mut().unwrap().1;
		}
		
		&mut next_steps.iter_mut().find(|s| s.0 == value).unwrap().1
	}


	fn make_step_param(step: &mut MatchStep) -> &mut MatchStep
	{
		if step.children_param.is_none()
		{
			let next_step = MatchStep::new();
			step.children_param = Some(Box::new(next_step));
			step.children_param.as_mut().unwrap()
		}
		else
		{
			step.children_param.as_mut().unwrap()
		}
	}


	pub fn parse_match(&self, parser: &mut Parser) -> Option<Match>
	{
		let mut exprs = Vec::new();
		
		match self.parse_match_step(parser, &self.root_step, &mut exprs)
		{
			Some(indices) =>
			{
				let result = Match
				{
					rule_indices: indices.iter().cloned().collect(),
					exprs: exprs
				};
				
				Some(result)
			}
			
			None => None
		}
	}
	
	
	fn parse_match_step<'s, 't: 's>(&'s self, parser: &mut Parser, step: &'t MatchStep, exprs: &mut Vec<Expression>) -> Option<&'t [usize]>
	{
		if !parser.next_is_linebreak()
		{
			// Try to match fixed tokens first, if some rule accepts that.
			let parser_state = parser.save(); 
			
			let c = parser.advance_partial();
			
			if let Some(ref next_steps) = step.children_exact.get(&c.to_ascii_lowercase())
			{
				for (ref value, ref next_step) in next_steps.iter()
				{
					if value.is_some()
						{ exprs.push(value.as_ref().unwrap().make_literal()); }
					
					if let Some(result) = self.parse_match_step(parser, next_step, exprs)
					{
						return Some(result);
					}
					
					if value.is_some()
						{ exprs.pop(); }
				}
			}
			
			parser.restore(parser_state);
			
			// Then try to match argument expressions, if some rule accepts that.
			if let Some(ref next_step) = step.children_param
			{
				let parser_state = parser.save();

				let expr = if parser.is_at_partial()
				{
					match parser.maybe_expect_partial_usize()
					{
						Some(value) => ExpressionValue::make_integer_from_usize(value).make_literal(),
						None => return None
					}
				}
				else
				{
					// Suppress reports for expression errors
					let report_original = std::mem::replace(&mut parser.report, RcReport::new());
					let maybe_expr = Expression::parse(parser);
					parser.report = report_original;

					match maybe_expr
					{
						Ok(expr) => expr,
						Err(()) => return None
					}
				};
				
				exprs.push(expr);
				
				if let Some(result) = self.parse_match_step(parser, &next_step, exprs)
					{ return Some(result); }
					
				exprs.pop();
				parser.restore(parser_state);
			}
		}
		
		// Finally, return a match if some rule ends here.
		if step.rule_indices.len() != 0
		{
			if parser.is_at_partial() || !parser.next_is_linebreak()
				{ return None }
			
			return Some(&step.rule_indices);
		}
		
		// Else, return no match.
		None
	}
	
	
	#[cfg(not(target_arch = "wasm32"))]
	pub fn print_debug(&self)
	{
		self.print_debug_inner(&self.root_step, 1);
	}
	
	
	#[cfg(not(target_arch = "wasm32"))]
	fn print_debug_inner(&self, step: &MatchStep, indent: usize)
	{
		for rule_index in &step.rule_indices
		{
			for _ in 0..indent
				{ print!("   "); }
				
			println!("match #{}", rule_index);
		}
			
		for (key, next_steps) in &step.children_exact
		{
			for (ref value, ref next_step) in next_steps
			{
				for _ in 0..indent
					{ print!("   "); }
				
				print!("{}", key);
				
				if value.is_some()
				{
					match &value.as_ref().unwrap()
					{
						&ExpressionValue::Integer(ref bigint) => print!(" (= {})", bigint),
						_ => unreachable!()
					}
				}
				
				println!();
				
				self.print_debug_inner(&next_step, indent + 1);
			}
		}
		
		if let Some(ref next_step) = step.children_param
		{
			for _ in 0..indent
				{ print!("   "); }
				
			println!("expr");
			self.print_debug_inner(&next_step, indent + 1);
		}
	}
}


impl MatchStep
{
	fn new() -> MatchStep
	{
		MatchStep
		{
			rule_indices: Vec::new(),
			children_exact: HashMap::new(),
			children_param: None
		}
	}
}