use crate::*;


pub struct Rule
{
    pub span: diagn::Span,
    pub pattern: Vec<PatternPart>,
    pub production: expr::Expression,
}


pub enum PatternPart
{
    Exact(char),
}


impl Rule
{
    pub fn new() -> Rule
    {
        Rule
        {
            span: diagn::Span::new_dummy(),
            pattern: Vec::new(),
            production: expr::Expression::new_dummy(),
        }
    }
	
	
	pub fn pattern_add_exact(&mut self, token: &syntax::Token)
	{
		for c in token.text().chars()
		{
			let part = PatternPart::Exact(c);
			self.pattern.push(part);
		}
	}
}