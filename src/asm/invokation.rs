use crate::*;


#[derive(Debug)]
pub struct Invokation
{
    pub ctx: asm::Context,
    pub size_guess: usize,
    pub span: diagn::Span,
    pub kind: InvokationKind,
}


#[derive(Debug)]
pub enum InvokationKind
{
    Rule(RuleInvokation),
    Data(DataInvokation),
    Label(LabelInvokation),
}


#[derive(Debug)]
pub struct RuleInvokation
{
    pub candidates: Vec<RuleInvokationCandidate>,
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
    NestedRuleset(Vec<RuleInvokationCandidate>),
}


#[derive(Debug)]
pub struct DataInvokation
{
    pub expr: expr::Expr,
    pub elem_size: Option<usize>,
}


#[derive(Debug)]
pub struct LabelInvokation;


impl Invokation
{
    pub fn get_rule_invok(&self) -> &RuleInvokation
    {
        if let InvokationKind::Rule(ref rule_invok) = self.kind
        {
            return rule_invok;
        }

        panic!();
    }


    pub fn get_data_invok(&self) -> &DataInvokation
    {
        if let InvokationKind::Data(ref data_invok) = self.kind
        {
            return data_invok;
        }

        panic!();
    }
}