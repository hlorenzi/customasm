use crate::*;


#[derive(Debug)]
pub struct RuleInvokation
{
    pub ctx: asm::Context,
    pub candidates: Vec<RuleInvokationCandidate>,
    pub span: diagn::Span,
}


#[derive(Debug)]
pub struct RuleInvokationCandidate
{
    pub rule_ref: asm::RuleRef,
    pub args: Vec<RuleInvokationArgument>,
}


#[derive(Debug)]
pub enum RuleInvokationArgument
{
    Expression(expr::Expr),
    NestedRule(Vec<RuleInvokationCandidate>),
}