use crate::*;


pub fn resolve_builtin_fn(
    name: &str)
    -> Option<fn(&mut expr::EvalFunctionInfo2) -> Result<expr::Value, ()>>
{
    match name.as_ref()
    {
        "assert" => Some(eval_builtin_assert),
        "le" => Some(eval_builtin_le),
        _ => None,
    }
}


pub fn get_static_size_builtin_fn(
    name: &str,
    info: &expr::StaticSizeInfo,
    args: &Vec<expr::Expr>)
    -> Option<usize>
{
    let get_static_size_fn = {
        match name.as_ref()
        {
            "le" => get_static_size_builtin_le,
            _ => return None,
        }
    };

    get_static_size_fn(
        info,
        args)
}


pub fn eval_builtin_fn(
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    let builtin_name = {
        match info.func
        {
            expr::Value::ExprBuiltInFunction(ref name) => name,
            _ => unreachable!(),
        }
    };

    let builtin_fn = resolve_builtin_fn(builtin_name).unwrap();
    builtin_fn(info)
}


pub fn eval_builtin_assert(
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let condition = info.args[0].value.expect_bool(
        info.report,
        info.args[0].span)?;

    if condition
    {
        Ok(expr::Value::Void)
    }
    else
    {
        let msg = diagn::Message::error_span(
            "assertion failed",
            info.span);
        
        Ok(expr::Value::FailedConstraint(
            info.report.wrap_in_parents_capped(msg)))
    }
}


pub fn eval_builtin_le(
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let bigint = info.args[0].value.expect_sized_bigint(
        info.report,
        info.args[0].span)?;
    
    if bigint.size.unwrap() % 8 != 0
    {
        info.report.push_parent(
            "argument to `le` must have a size multiple of 8",
            info.args[0].span);

        info.report.note(format!(
            "got size {}",
            bigint.size.unwrap()));

        info.report.pop_parent();
        
        return Err(());
    }

    Ok(expr::Value::make_integer(bigint.convert_le()))
}


pub fn get_static_size_builtin_le(
    info: &expr::StaticSizeInfo,
    args: &Vec<expr::Expr>)
    -> Option<usize>
{
    if args.len() == 1
    {
        args[0].get_static_size(info)
    }
    else
    {
        None
    }
}