use crate::*;


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExprBuiltinFn
{
    Assert,
    Sizeof,
    Le,
    Ascii,
    Utf8,
    Utf16be,
    Utf16le,
    Utf32be,
    Utf32le,
    Strlen,
}


pub fn resolve_builtin_fn(
    name: &str,
    opts: &asm::AssemblyOptions)
    -> Option<ExprBuiltinFn>
{
    if !opts.use_legacy_behavior
    {
        match name
        {
            "$assert" => Some(ExprBuiltinFn::Assert),
            "$sizeof" => Some(ExprBuiltinFn::Sizeof),
            "$le" => Some(ExprBuiltinFn::Le),
            "$ascii" => Some(ExprBuiltinFn::Ascii),
            "$utf8" => Some(ExprBuiltinFn::Utf8),
            "$utf16be" => Some(ExprBuiltinFn::Utf16be),
            "$utf16le" => Some(ExprBuiltinFn::Utf16le),
            "$utf32be" => Some(ExprBuiltinFn::Utf32be),
            "$utf32le" => Some(ExprBuiltinFn::Utf32le),
            "$strlen" => Some(ExprBuiltinFn::Strlen),
            _ => return None,
        }
    }
    else
    {
        match name
        {
            "assert" => Some(ExprBuiltinFn::Assert),
            "sizeof" => Some(ExprBuiltinFn::Sizeof),
            "le" => Some(ExprBuiltinFn::Le),
            "ascii" => Some(ExprBuiltinFn::Ascii),
            "utf8" => Some(ExprBuiltinFn::Utf8),
            "utf16be" => Some(ExprBuiltinFn::Utf16be),
            "utf16le" => Some(ExprBuiltinFn::Utf16le),
            "utf32be" => Some(ExprBuiltinFn::Utf32be),
            "utf32le" => Some(ExprBuiltinFn::Utf32le),
            "strlen" => Some(ExprBuiltinFn::Strlen),
            _ => return None,
        }
    }
}


pub fn get_builtin_fn_eval(
    builtin_fn: ExprBuiltinFn)
    -> fn(&mut expr::EvalFunctionQuery) -> Result<expr::Value, ()>
{
    match builtin_fn
    {
        ExprBuiltinFn::Assert => eval_builtin_assert,
        ExprBuiltinFn::Sizeof => eval_builtin_sizeof,
        ExprBuiltinFn::Le => eval_builtin_le,
        ExprBuiltinFn::Ascii => eval_builtin_ascii,
        ExprBuiltinFn::Utf8 => eval_builtin_utf8,
        ExprBuiltinFn::Utf16be => eval_builtin_utf16be,
        ExprBuiltinFn::Utf16le => eval_builtin_utf16le,
        ExprBuiltinFn::Utf32be => eval_builtin_utf32be,
        ExprBuiltinFn::Utf32le => eval_builtin_utf32le,
        ExprBuiltinFn::Strlen => eval_builtin_strlen,
    }
}


pub fn get_static_size_builtin_fn(
    builtin_fn: ExprBuiltinFn,
    provider: &expr::StaticallyKnownProvider,
    args: &Vec<expr::Expr>)
    -> Option<usize>
{
    let get_static_size_fn = {
        match builtin_fn
        {
            ExprBuiltinFn::Sizeof => get_static_size_builtin_sizeof,
            ExprBuiltinFn::Le => get_static_size_builtin_le,
            _ => return None,
        }
    };

    get_static_size_fn(
        provider,
        args)
}


pub fn get_statically_known_value_builtin_fn(
    builtin_fn: ExprBuiltinFn,
    _args: &Vec<expr::Expr>)
    -> bool
{
    match builtin_fn
    {
        ExprBuiltinFn::Assert => false,
        ExprBuiltinFn::Sizeof => true,
        ExprBuiltinFn::Le => true,
        ExprBuiltinFn::Ascii => true,
        ExprBuiltinFn::Utf8 => true,
        ExprBuiltinFn::Utf16be => true,
        ExprBuiltinFn::Utf16le => true,
        ExprBuiltinFn::Utf32be => true,
        ExprBuiltinFn::Utf32le => true,
        ExprBuiltinFn::Strlen => true,
    }
}


pub fn eval_builtin_assert(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_min_max_arg_number(1, 2)?;

    let condition = query.args[0]
        .value
        .expect_bool(
            query.report,
            query.args[0].span)?;

    if !condition
    {
        let msg = {
            if query.args.len() == 2
            {
                diagn::Message::error_span(
                    format!(
                        "assertion failed: {}",
                        query.args[1]
                            .value
                            .expect_string(query.report, query.args[1].span)?
                            .utf8_contents),
                    query.span)
            }
            else {
                diagn::Message::error_span("assertion failed", query.span)
            }
        };
        
        return Ok(expr::Value::FailedConstraint(
            expr::Value::make_metadata(),
            query.report.wrap_in_parents_capped(msg)));
    }

    Ok(expr::Value::make_void())
}


pub fn eval_builtin_sizeof(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_arg_number(1)?;

    let (_bigint, size) = query.args[0].value.expect_sized_integerlike(
        query.report,
        query.args[0].span)?;
    
    Ok(expr::Value::make_integer(size))
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


pub fn get_static_size_builtin_sizeof(
    provider: &expr::StaticallyKnownProvider,
    args: &Vec<expr::Expr>)
    -> Option<usize>
{
    if args.len() == 1
    {
        args[0].get_static_size(provider)
    }
    else
    {
        None
    }
}


pub fn get_static_size_builtin_le(
    provider: &expr::StaticallyKnownProvider,
    args: &Vec<expr::Expr>)
    -> Option<usize>
{
    if args.len() == 1
    {
        args[0].get_static_size(provider)
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


pub fn eval_builtin_strlen(
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_arg_number(1)?;

    let s = query.args[0].value.expect_string(
        query.report,
        query.args[0].span)?;

    Ok(expr::Value::make_integer(s.utf8_contents.len()))
}