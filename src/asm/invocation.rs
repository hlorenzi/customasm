use crate::*;


#[derive(Debug)]
pub struct Invocation
{
    pub ctx: asm::Context,
    pub size_guess: usize,
    pub span: diagn::Span,
    pub kind: InvocationKind,
}


#[derive(Debug)]
pub enum InvocationKind
{
    Rule(RuleInvocation),
    Data(DataInvocation),
    Label(LabelInvocation),
}


#[derive(Debug)]
pub struct RuleInvocation
{
    pub candidates: Vec<RuleInvocationCandidate>,
}


#[derive(Debug)]
pub struct RuleInvocationCandidate
{
    pub rule_ref: asm::RuleRef,
    pub args: Vec<RuleInvocationArgument>,
}


#[derive(Debug)]
pub enum RuleInvocationArgument
{
    Expression(expr::Expr),
    NestedRuleset(Vec<RuleInvocationCandidate>),
}


#[derive(Debug)]
pub struct DataInvocation
{
    pub expr: expr::Expr,
    pub elem_size: Option<usize>,
}


#[derive(Debug)]
pub struct LabelInvocation;


impl Invocation
{
    pub fn get_rule_invoc(&self) -> &RuleInvocation
    {
        if let InvocationKind::Rule(ref rule_invoc) = self.kind
        {
            return rule_invoc;
        }

        panic!();
    }


    pub fn get_data_invoc(&self) -> &DataInvocation
    {
        if let InvocationKind::Data(ref data_invoc) = self.kind
        {
            return data_invoc;
        }

        panic!();
    }
}