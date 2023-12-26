use crate::*;

#[derive(Debug)]
pub struct Ruledef
{
    pub item_ref: util::ItemRef<Self>,
    pub is_subruledef: bool,
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule
{
    pub pattern_span: diagn::Span,
    pub pattern: RulePattern,

    /// Used in instruction-matching to prioritize matches
    /// with more "exact" pattern-parts
    pub exact_part_count: usize,

    pub parameters: Vec<RuleParameter>,
    pub expr: expr::Expr,
}

pub type RulePattern = Vec<RulePatternPart>;

#[derive(Debug)]
pub enum RulePatternPart
{
    Whitespace,
    Exact(char),
    ParameterIndex(usize),
}

#[derive(Debug)]
pub struct RuleParameter
{
    pub name: String,
    pub typ: RuleParameterType,
}

#[derive(Copy, Clone, Debug)]
pub enum RuleParameterType
{
    Unspecified,
    RuledefRef(util::ItemRef<Ruledef>),
    Unsigned(usize),
    Signed(usize),
    Integer(usize),
}

pub fn define(
    report: &mut diagn::Report,
    ast: &asm::AstTopLevel,
    decls: &mut asm::ItemDecls,
    defs: &mut asm::ItemDefs,
) -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        if let asm::AstAny::DirectiveRuledef(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();

            let mut rules = Vec::new();

            for node_rule in &node.rules
            {
                let rule = resolve_rule(report, decls, &node_rule)?;

                rules.push(rule);
            }

            let ruledef = Ruledef {
                item_ref,
                is_subruledef: node.is_subruledef,
                rules,
            };

            defs.ruledefs.define(item_ref, ruledef);
        }
    }

    Ok(())
}

pub fn resolve_rule(
    report: &mut diagn::Report,
    decls: &mut asm::ItemDecls,
    ast_rule: &asm::AstRule,
) -> Result<Rule, ()>
{
    let mut pattern = RulePattern::new();
    let mut exact_parts = 0;
    let mut parameters = Vec::<RuleParameter>::new();

    for ast_part in &ast_rule.pattern
    {
        let part = {
            match &ast_part
            {
                asm::AstRulePatternPart::Whitespace => RulePatternPart::Whitespace,

                asm::AstRulePatternPart::Exact(c) =>
                {
                    exact_parts += 1;
                    RulePatternPart::Exact(*c)
                }

                asm::AstRulePatternPart::Parameter(ast_param) =>
                {
                    let param_index =
                        resolve_rule_parameter(report, decls, &mut parameters, &ast_param)?;

                    RulePatternPart::ParameterIndex(param_index)
                }
            }
        };

        pattern.push(part);
    }

    Ok(Rule {
        pattern_span: ast_rule.pattern_span,
        pattern,
        exact_part_count: exact_parts,
        parameters,
        expr: ast_rule.expr.clone(),
    })
}

pub fn resolve_rule_parameter(
    report: &mut diagn::Report,
    decls: &mut asm::ItemDecls,
    parameters: &mut Vec<RuleParameter>,
    ast_param: &asm::AstRuleParameter,
) -> Result<usize, ()>
{
    let typ = {
        match &ast_param.typ
        {
            asm::AstRuleParameterType::Unspecified => RuleParameterType::Unspecified,

            asm::AstRuleParameterType::Integer(i) => RuleParameterType::Integer(*i),

            asm::AstRuleParameterType::Unsigned(u) => RuleParameterType::Unsigned(*u),

            asm::AstRuleParameterType::Signed(s) => RuleParameterType::Signed(*s),

            asm::AstRuleParameterType::Ruledef(ruledef_name) =>
            {
                let item_ref = decls.ruledefs.get_by_name_global(
                    report,
                    ast_param.type_span,
                    &ruledef_name,
                )?;

                decls.ruledefs.add_span_ref(ast_param.type_span, item_ref);

                RuleParameterType::RuledefRef(item_ref)
            }
        }
    };

    let name = ast_param.name.clone();

    let maybe_duplicate_param = parameters.iter().find(|param| param.name == name);

    if let Some(_) = maybe_duplicate_param
    {
        report.error_span(
            format!("duplicate parameter `{}`", name),
            ast_param.name_span,
        );

        return Err(());
    }

    let param_index = parameters.len();
    let param = RuleParameter { name, typ };

    parameters.push(param);
    Ok(param_index)
}

impl Ruledef
{
    pub fn get_rule(&self, rule_ref: util::ItemRef<Rule>) -> &Rule
    {
        &self.rules[rule_ref.0]
    }

    pub fn iter_rule_refs(&self) -> impl Iterator<Item = util::ItemRef<Rule>>
    {
        (0..self.rules.len())
            .into_iter()
            .map(|i| util::ItemRef::<Rule>::new(i))
    }
}
