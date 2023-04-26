use crate::*;


pub fn resolve_builtin_fn(
    name: &str)
    -> Option<fn(&mut expr::EvalFunctionQuery) -> Result<expr::Value, ()>>
{
    match name.as_ref()
    {
        "assert" => Some(eval_builtin_assert),
        "le" => Some(eval_builtin_le),
        "ascii" => Some(eval_builtin_ascii),
        "utf8" => Some(eval_builtin_utf8),
        "utf16be" => Some(eval_builtin_utf16be),
        "utf16le" => Some(eval_builtin_utf16le),
        "utf32be" => Some(eval_builtin_utf32be),
        "utf32le" => Some(eval_builtin_utf32le),
        _ => None,
    }
}


pub fn get_static_size_builtin_fn(
    name: &str,
    info: &expr::StaticallyKnownProvider,
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


pub fn get_statically_known_value_builtin_fn(
    name: &str,
    info: &expr::StaticallyKnownProvider,
    args: &Vec<expr::Expr>)
    -> bool
{
    match name.as_ref()
    {
        "le" => get_statically_known_value_builtin_le(info, args),
        _ => false,
    }
}


pub fn eval_builtin_fn(
    info: &mut expr::EvalFunctionQuery)
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
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_arg_number(1)?;

    let condition = query.args[0].value.expect_bool(
        query.report,
        query.args[0].span)?;

    if condition
    {
        Ok(expr::Value::Void)
    }
    else
    {
        let msg = diagn::Message::error_span(
            "assertion failed",
            query.span);
        
        Ok(expr::Value::FailedConstraint(
            query.report.wrap_in_parents_capped(msg)))
    }
}


pub fn eval_builtin_le(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_arg_number(1)?;

    let bigint = query.args[0].value.expect_sized_bigint(
        query.report,
        query.args[0].span)?;
    
    if bigint.size.unwrap() % 8 != 0
    {
        query.report.push_parent(
            "argument to `le` must have a size multiple of 8",
            query.args[0].span);

        query.report.note(format!(
            "got size {}",
            bigint.size.unwrap()));

        query.report.pop_parent();
        
        return Err(());
    }

    Ok(expr::Value::make_integer(bigint.convert_le()))
}


pub fn get_statically_known_value_builtin_le(
    info: &expr::StaticallyKnownProvider,
    args: &Vec<expr::Expr>)
    -> bool
{
    if args.len() == 1
    {
        args[0].is_value_statically_known(info)
    }
    else
    {
        false
    }
}


pub fn get_static_size_builtin_le(
    info: &expr::StaticallyKnownProvider,
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


pub fn eval_builtin_string_encoding(
    encoding: &str,
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_arg_number(1)?;

    let s = query.args[0].value.expect_string(
        query.report,
        query.args[0].span)?;

    Ok(expr::Value::make_string(
        &s.utf8_contents,
        encoding))
}


pub fn eval_builtin_ascii(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_string_encoding("ascii", query)
}


pub fn eval_builtin_utf8(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_string_encoding("utf8", query)
}


pub fn eval_builtin_utf16be(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_string_encoding("utf16be", query)
}


pub fn eval_builtin_utf16le(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_string_encoding("utf16le", query)
}


pub fn eval_builtin_utf32be(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_string_encoding("utf32be", query)
}


pub fn eval_builtin_utf32le(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_string_encoding("utf32le", query)
}