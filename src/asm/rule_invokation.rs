use crate::*;


#[derive(Debug)]
pub struct RuleInvokation
{
    pub bit_offset: usize,
    pub candidates: Vec<RuleInvokationCandidate>,
}


#[derive(Debug)]
pub struct RuleInvokationCandidate
{
    pub rule_ref: asm::RuleRef,
    pub args: Vec<RuleInvokationCandidateArgument>,
}


#[derive(Debug)]
pub enum RuleInvokationCandidateArgument
{
    Expression(expr::Expression),
    RuleGroup(Vec<RuleInvokationCandidate>),
}