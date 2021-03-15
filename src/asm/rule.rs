use crate::*;


#[derive(Debug)]
pub struct Rule
{
    pub span: diagn::Span,
    pub pattern: Vec<PatternPart>,
    pub parameters: Vec<PatternParameter>,
    pub production: expr::Expr,
}


#[derive(Debug)]
pub enum PatternPart
{
    Exact(char),
    Parameter(usize),
}


#[derive(Debug)]
pub struct PatternParameter
{
    pub name: String,
    pub typ: PatternParameterType
}


#[derive(Copy, Clone, Debug)]
pub enum PatternParameterType
{
    Unspecified,
    Ruleset(asm::RulesetRef),
    Unsigned(usize),
    Signed(usize),
    Integer(usize),
}


impl Rule
{
    pub fn new() -> Rule
    {
        Rule
        {
            span: diagn::Span::new_dummy(),
            pattern: Vec::new(),
            parameters: Vec::new(),
            production: expr::Expr::new_dummy(),
        }
    }
	
	
	pub fn pattern_add_exact(&mut self, token: &syntax::Token)
	{
		for c in token.text().chars()
		{
			let part = PatternPart::Exact(c.to_ascii_lowercase());
			self.pattern.push(part);
		}
	}
	
	
	pub fn pattern_add_parameter(&mut self, param: PatternParameter)
	{
        let param_index = self.parameters.len();
        self.parameters.push(param);
		self.pattern.push(PatternPart::Parameter(param_index));
	}


    pub fn get_specificity_score(&self) -> usize
    {
        let mut count = 0;

        for part in &self.pattern
        {
            if let PatternPart::Exact(_) = part
            {
                count += 1;
            }
        }

        count
    }
}