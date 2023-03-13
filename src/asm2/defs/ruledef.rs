use super::*;


#[derive(Debug)]
pub struct Ruledef
{
    pub name_span: diagn::Span,
    pub name: String,
    pub rules: Vec<Rule>,
}


#[derive(Debug)]
pub struct Rule
{
    pub pattern_span: diagn::Span,
    pub pattern: RulePattern,
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


#[derive(Debug)]
pub enum RuleParameterType
{
    Unspecified,
    RuledefRef(asm2::ItemRef<Ruledef>),
    Unsigned(usize),
    Signed(usize),
    Integer(usize),
}


pub fn resolve(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &mut asm2::ItemDecls,
    defs: &mut ItemDefs)
    -> Result<(), ()>
{
    for any_node in &ast.nodes
    {
        if let asm2::AstAny::DirectiveRuledef(node) = any_node
        {
            let item_ref = node.item_ref.unwrap();
            
            let mut rules = Vec::new();

            for node_rule in &node.rules
            {
                let rule = resolve_rule(
                    report,
                    decls,
                    &node_rule)?;

                rules.push(rule);
            }

            let decl = decls.ruledefs.get(item_ref);
            let ruledef = Ruledef {
                name_span: decl.span.clone(),
                name: decl.name.clone(),
                rules,
            };

            defs.ruledefs.set(item_ref, ruledef);
        }
    }


    Ok(())
}


pub fn resolve_rule(
    report: &mut diagn::Report,
    decls: &mut asm2::ItemDecls,
    ast_rule: &asm2::AstRule)
    -> Result<Rule, ()>
{
    let mut pattern = RulePattern::new();
    let mut parameters = Vec::<RuleParameter>::new();

    for ast_part in &ast_rule.pattern
    {
        let part = {
            match &ast_part
            {
                asm2::AstRulePatternPart::Whitespace =>
                    RulePatternPart::Whitespace,
                    
                asm2::AstRulePatternPart::Exact(c) =>
                    RulePatternPart::Exact(*c),
                
                asm2::AstRulePatternPart::Parameter(ast_param) =>
                {
                    let param_index = resolve_rule_parameter(
                        report,
                        decls,
                        &mut parameters,
                        &ast_param)?;

                    RulePatternPart::ParameterIndex(param_index)
                }
            }
        };

        pattern.push(part);
    }

    Ok(Rule {
        pattern_span: ast_rule.pattern_span.clone(),
        pattern,
        parameters,
        expr: ast_rule.expr.clone(),
    })
}


pub fn resolve_rule_parameter(
    report: &mut diagn::Report,
    decls: &mut asm2::ItemDecls,
    parameters: &mut Vec::<RuleParameter>,
    ast_param: &asm2::AstRuleParameter)
    -> Result<usize, ()>
{
    let typ = {
        match &ast_param.typ
        {
            asm2::AstRuleParameterType::Unspecified =>
                RuleParameterType::Unspecified,
                
            asm2::AstRuleParameterType::Integer(i) =>
                RuleParameterType::Integer(*i),
            
            asm2::AstRuleParameterType::Unsigned(u) =>
                RuleParameterType::Unsigned(*u),
                
            asm2::AstRuleParameterType::Signed(s) =>
                RuleParameterType::Signed(*s),
            
            asm2::AstRuleParameterType::Ruledef(ruledef_name) =>
            {
                if let Some(item_ref) = decls.ruledefs.get_from_name(&ruledef_name)
                {
                    decls.ruledefs.add_span_ref(
                        ast_param.type_span.clone(),
                        item_ref);
                    
                    RuleParameterType::RuledefRef(item_ref)
                }
                else
                {
                    report.error_span(
                        format!("unknown ruledef `{}`", ruledef_name),
                        &ast_param.type_span);
                    
                    return Err(());
                }
            }
        }
    };

    let name = ast_param.name.clone();


    let maybe_duplicate_param = parameters
        .iter()
        .find(|param| param.name == name);

    if let Some(_) = maybe_duplicate_param
    {
        report.error_span(
            format!("duplicate parameter `{}`", name),
            &ast_param.name_span);
        
        return Err(());
    }


    let param_index = parameters.len();
    let param = RuleParameter {
        name,
        typ,
    };

    parameters.push(param);
    Ok(param_index)
}