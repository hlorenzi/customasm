use crate::*;


pub fn resolve_ifs(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast: &mut asm::AstTopLevel,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs)
    -> Result<usize, ()>
{
    let mut resolved_count = 0;


    for n in (0..ast.nodes.len()).rev()
    {
        let asm::AstAny::DirectiveIf(node) = &ast.nodes[n]
            else { continue };
        
        let condition_result = 
            asm::resolver::eval_simple(
                report,
                opts,
                decls,
                defs,
                &node.condition_expr)?;

        let expr::Value::Bool(_, condition_result) = condition_result
            else { continue };
            
        if opts.debug_iterations
        {
            println!("  #if: {} = {}",
                fileserver.get_excerpt(node.condition_expr.span()),
                condition_result);
        }

        let asm::AstAny::DirectiveIf(node) = ast.nodes.remove(n)
            else { unreachable!() };
        
        if condition_result
        {
            ast.nodes.splice(
                n..n,
                node.true_arm.nodes);
        }
        else if let Some(false_arm) = node.false_arm
        {
            ast.nodes.splice(
                n..n,
                false_arm.nodes);
        }

        resolved_count += 1;
    }

    Ok(resolved_count)
}


pub fn check_leftover_ifs(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast: &asm::AstTopLevel,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs)
    -> Result<(), ()>
{
    for node in &ast.nodes
    {
        let asm::AstAny::DirectiveIf(node) = node
            else { continue };

        report.push_parent(
            "unresolved condition",
            node.condition_expr.span());
        
        let condition_result = 
            asm::resolver::eval_certain(
                report,
                opts,
                decls,
                defs,
                &node.condition_expr);

        report.pop_parent();

        if let Ok(_) = condition_result
        {
            report.error_span(
                "unresolved condition",
                node.condition_expr.span());
        }

        return Err(());
    }

    Ok(())
}