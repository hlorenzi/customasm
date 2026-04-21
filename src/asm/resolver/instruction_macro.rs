use crate::*;


pub fn resolve_instruction_macros(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast: &mut asm::AstTopLevel,
    _decls: &asm::ItemDecls,
    defs: &asm::ItemDefs)
    -> Result<usize, ()>
{
    let mut resolved_count = 0;


    for n in (0..ast.nodes.len()).rev()
    {
        let asm::AstAny::Instruction(node) = &ast.nodes[n]
            else { continue };

        let instr = defs.instructions.get(node.item_ref.unwrap());

        let match_is_macro = |m: &asm::InstructionMatch| {
            let ruledef = defs.ruledefs.get(m.ruledef_ref);
            let rule = ruledef.get_rule(m.rule_ref);
            matches!(rule.production, asm::AstRuleProduction::Macro(_))
        };

        if !instr.matches.iter().any(match_is_macro)
            { continue; }

        if instr.matches.len() != 1
        {
            report.error_span("multiple matches for macro instruction", node.span);
            return Err(());
        }

        let macro_match = &instr.matches[0];
        let ruledef = defs.ruledefs.get(macro_match.ruledef_ref);
        let rule = ruledef.get_rule(macro_match.rule_ref);
        let asm::AstRuleProduction::Macro(ref instr_macro) = rule.production
            else { unreachable!() };
        
        if opts.debug_iterations
        {
            println!("macro `{}`", &node.src);
        }

        let asm::AstAny::Instruction(_node) = ast.nodes.remove(n)
            else { unreachable!() };
        
        ast.nodes.splice(
            n..n,
            instr_macro.nodes.clone());

        resolved_count += 1;
    }

    Ok(resolved_count)
}