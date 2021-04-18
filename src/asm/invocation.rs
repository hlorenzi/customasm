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


#[derive(Clone, Debug)]
pub struct RuleInvocationCandidate
{
    pub rule_ref: asm::RuleRef,
    pub specificity: usize,
    pub args: Vec<RuleInvocationArgument>,
    pub token_args: Vec<Option<Vec<syntax::Token>>>,
}


#[derive(Clone, Debug)]
pub enum RuleInvocationArgument
{
    Expression(expr::Expr),
    NestedRuleset(RuleInvocationCandidate),
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


impl RuleInvocationCandidate
{
    pub fn calculate_specificity_score(&self, asm_state: &asm::State) -> usize
    {
        let rule_group = &asm_state.rulesets[self.rule_ref.ruleset_ref.index];
        let rule = &rule_group.rules[self.rule_ref.index];
        let mut score = rule.get_specificity_score();

        for arg in &self.args
        {
            if let RuleInvocationArgument::NestedRuleset(nested) = arg
            {
                score += nested.calculate_specificity_score(asm_state);
            }
        }

        score
    }
}