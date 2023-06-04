use crate::*;


pub fn eval(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    eval_ctx: &mut expr::EvalContext,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut provider = |query: expr::EvalQuery|
    {
        match query
        {
            expr::EvalQuery::Variable(query_var) =>
                asm::resolver::eval_variable(
                    decls,
                    defs,
                    ctx,
                    query_var),
                    
            expr::EvalQuery::RelativeLabel(query_rel_label) =>
                asm::resolver::eval_relative_label(
                    decls,
                    defs,
                    ctx,
                    query_rel_label),
                    
            expr::EvalQuery::Function(query_fn) =>
                asm::resolver::eval_fn(
                    opts,
                    fileserver,
                    decls,
                    defs,
                    ctx,
                    query_fn),
                
            expr::EvalQuery::AsmBlock(query_asm) =>
                asm::resolver::eval_asm(
                    opts,
                    fileserver,
                    decls,
                    defs,
                    ctx,
                    query_asm),
        }
    };

    expr.eval_with_ctx(
        report,
        eval_ctx,
        &mut provider)
}


pub fn eval_simple(
    report: &mut diagn::Report,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut provider = |query: expr::EvalQuery|
    {
        match query
        {
            expr::EvalQuery::Variable(query_var) =>
                asm::resolver::eval_variable_simple(
                    decls,
                    defs,
                    query_var),
                    
            expr::EvalQuery::RelativeLabel(query_rel_label) =>
                expr::dummy_eval_rel_label(query_rel_label),
                
            expr::EvalQuery::Function(query_fn) =>
                expr::dummy_eval_fn(query_fn),
                
            expr::EvalQuery::AsmBlock(query_asm) =>
                expr::dummy_eval_asm(query_asm),
        }
    };

    let result = expr.eval_with_ctx(
        report,
        &mut expr::EvalContext::new(),
        &mut provider)?;

    match result
    {
        expr::Value::Unknown =>
        {
            report.error_span(
                "cannot resolve expression",
                expr.span());
    
            Err(())
        }

        expr::Value::FailedConstraint(msg) =>
        {
            report.message(msg);
            Err(())
        }

        _ => Ok(result)
    }
}