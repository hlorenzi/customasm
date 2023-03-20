use crate::*;


pub fn resolve_builtin(
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


pub fn eval_builtin(
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    let builtin_name = {
        match info.func
        {
            expr::Value::BuiltInFunction(ref name) => name,
            _ => unreachable!(),
        }
    };

    let builtin_fn = resolve_builtin(builtin_name).unwrap();
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
        info.report.error_span(
            "assertion failed",
            info.span);
        
        Ok(expr::Value::FailedConstraint)
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
        info.report.error_span(
            format!(
                "argument to `le` must have a size multiple of 8 (got size {})",
                bigint.size.unwrap()),
            info.span);
        
        return Err(());
    }

    Ok(expr::Value::make_integer(bigint.convert_le()))
}