use diagn::RcReport;
use syntax::{TokenKind, Parser};
use expr::{Expression, ExpressionValue};
use asm::cpudef::{Rule, RuleParameterType, RulePatternPart, CustomTokenDef};
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
	children_exact: HashMap<MatchStepExact, Vec<(Option<ExpressionValue>, MatchStep)>>,
	children_param: HashMap<MatchStepParameter, MatchStep>
}


#[derive(Debug, Eq, PartialEq, Hash)]
struct MatchStepExact(TokenKind, Option<String>);


#[derive(Debug, Eq, PartialEq, Hash)]
struct MatchStepParameter;


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
			{ RulePatternMatcher::build_step(report.clone(), &mut root_step, &rules[i], &rules[i].pattern_parts, i, custom_token_defs)?; }
		
		
		Ok(RulePatternMatcher
		{
			root_step: root_step
		})
	}
	

	fn build_step(report: RcReport, step: &mut MatchStep, rule: &Rule, next_parts: &[RulePatternPart], rule_index: usize, custom_token_defs: &Vec<CustomTokenDef>) -> Result<(), ()>
	{
		if next_parts.len() == 0
		{
			step.rule_indices.push(rule_index);
			step.rule_indices.sort();
			return Ok(());
		}
		
		match next_parts[0]
		{
			RulePatternPart::Exact(kind, ref excerpt) =>
			{
				let step_kind = MatchStepExact(kind, excerpt.as_ref().map(|s| s.to_ascii_lowercase()));
				
				{
					let mut maybe_next_steps = step.children_exact.get_mut(&step_kind);
					
					if let Some(ref mut next_steps) = maybe_next_steps
					{
						if let Some(&mut (_, ref mut next_step)) = next_steps.iter_mut().find(|s| s.0 == None)
						{
							RulePatternMatcher::build_step(report.clone(), next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
							return Ok(());
						}
						
						let mut next_step = MatchStep::new();
						RulePatternMatcher::build_step(report.clone(), &mut next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
						next_steps.push((None, next_step));
						return Ok(());
					}
				}
				
				let mut next_step = MatchStep::new();
				RulePatternMatcher::build_step(report.clone(), &mut next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
				step.children_exact.insert(step_kind, vec![(None, next_step)]);
			}
			
			RulePatternPart::Parameter(param_index) =>
			{
				if let RuleParameterType::CustomTokenDef(tokendef_index) = rule.params[param_index].typ
				{
					let custom_token_def = &custom_token_defs[tokendef_index];
					
					for (excerpt, value) in &custom_token_def.excerpt_to_value_map
					{
						let step_kind = MatchStepExact(TokenKind::Identifier, Some(excerpt.to_ascii_lowercase()));
						
						{
							let mut maybe_next_steps = step.children_exact.get_mut(&step_kind);
							
							if let Some(ref mut next_steps) = maybe_next_steps
							{
								if let Some(&mut (_, ref mut next_step)) = next_steps.iter_mut().find(|s| s.0.as_ref() == Some(value))
								{
									RulePatternMatcher::build_step(report.clone(), next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
									continue;
								}
								
								let mut next_step = MatchStep::new();
								RulePatternMatcher::build_step(report.clone(), &mut next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
								next_steps.push((Some(value.clone()), next_step));
								continue;
							}
						}
						
						let mut next_step = MatchStep::new();
						RulePatternMatcher::build_step(report.clone(), &mut next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
						step.children_exact.insert(step_kind, vec![(Some(value.clone()), next_step)]);
					}
				}
				else
				{
					let step_kind = MatchStepParameter;
					
					if let Some(next_step) = step.children_param.get_mut(&step_kind)
						{ return RulePatternMatcher::build_step(report.clone(), next_step, rule, &next_parts[1..], rule_index, custom_token_defs); }
					
					let mut next_step = MatchStep::new();
					RulePatternMatcher::build_step(report.clone(), &mut next_step, rule, &next_parts[1..], rule_index, custom_token_defs)?;
					step.children_param.insert(step_kind, next_step);
				}
			}
		}
		
		return Ok(());
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
			
			let tk = parser.advance();
			
			let step_exact = MatchStepExact(tk.kind, tk.excerpt.map(|s| s.to_ascii_lowercase()));
			
			if let Some(ref next_steps) = step.children_exact.get(&step_exact)
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
			if let Some(next_step) = step.children_param.get(&MatchStepParameter)
			{
				let parser_state = parser.save(); 
				
				let expr = match Expression::parse(parser)
				{
					Ok(expr) => expr,
					Err(()) => return None
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
			if !parser.next_is_linebreak()
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
				
				print!("{}", key.0.printable_excerpt(key.1.as_ref().map(|s| s as &str)));
				
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
		
		for (_, next_step) in &step.children_param
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
			children_param: HashMap::new()
		}
	}
}