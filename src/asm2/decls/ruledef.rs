use super::*;


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel,
    decls: &mut ItemDecls)
    -> Result<(), ()>
{
    for any_node in &mut ast.nodes
    {
        if let asm2::AstAny::DirectiveRuledef(ref mut node) = any_node
        {
            let name = node.name
                .clone()
                .unwrap_or_else(||
                    format!(
                        "#anonymous_ruledef_{}",
                        decls.ruledefs.name_map.len()));


            if let Some(&ruledef_ref) = decls.ruledefs.name_map.get(&name)
            {
                let prev_decl = decls.ruledefs.get(ruledef_ref);
                
                report.push_parent(
                    format!("duplicate ruledef `{}`", name),
                    &node.name_span);
                
                report.note_span(
                    "first declared here",
                    &prev_decl.span);

                report.pop_parent();
                continue;
            }

            
            let item_ref = decls.ruledefs.register(
                name.clone(),
                node.name_span.clone());

            node.item_ref = Some(item_ref);

            decls.ruledefs.add_span_ref(
                node.name_span.clone(),
                item_ref);
        }
    }


    Ok(())
}