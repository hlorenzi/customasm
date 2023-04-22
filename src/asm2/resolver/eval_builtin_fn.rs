use crate::*;


type BuiltinFn = fn(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>;


pub fn resolve_builtin_fn(
    name: &str)
    -> Option<BuiltinFn>
{
    match name
    {
        "incbin" => Some(eval_builtin_incbin),
        "incbinstr" => Some(eval_builtin_incbinstr),
        "inchexstr" => Some(eval_builtin_inchexstr),
        _ => None,
    }
}


pub fn eval_builtin_fn(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    if let expr::Value::AsmBuiltInFunction(ref name) = info.func
    {
        let builtin_fn = resolve_builtin_fn(name).unwrap();

        builtin_fn(
            fileserver,
            decls,
            defs,
            ctx,
            info)
    }
    else
    {
        unreachable!()
    }
}


fn eval_builtin_incbin(
    fileserver: &dyn util::FileServer,
    _decls: &asm2::ItemDecls,
    _defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let relative_filename = info.args[0].value.expect_string(
        info.report,
        info.args[0].span)?;

    let absolute_filename = util::filename_navigate2(
        info.report,
        &ctx.filename_ctx.unwrap(),
        &relative_filename.utf8_contents,
        &info.span)?;

    let bytes = fileserver.get_bytes2(
        info.report,
        Some(&info.span),
        &absolute_filename)?;

    Ok(expr::Value::make_integer(
        util::BigInt::from_bytes_be(&bytes)))
}


fn eval_builtin_incbinstr(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    eval_builtin_incstr(
        fileserver,
        1,
        decls,
        defs,
        ctx,
        info)
}


fn eval_builtin_inchexstr(
    fileserver: &dyn util::FileServer,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    eval_builtin_incstr(
        fileserver,
        4,
        decls,
        defs,
        ctx,
        info)
}


fn eval_builtin_incstr(
    fileserver: &dyn util::FileServer,
    bits_per_char: usize,
    _decls: &asm2::ItemDecls,
    _defs: &asm2::ItemDefs,
    ctx: &asm2::ResolverContext,
    info: &mut expr::EvalFunctionInfo2)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let relative_filename = info.args[0].value.expect_string(
        info.report,
        info.args[0].span)?;

    let absolute_filename = util::filename_navigate2(
        info.report,
        &ctx.filename_ctx.unwrap(),
        &relative_filename.utf8_contents,
        &info.span)?;

    let chars = fileserver.get_chars2(
        info.report,
        Some(&info.span),
        &absolute_filename)?;

    
    let mut bitvec = util::BitVec::new();

    for c in chars
    {
        if syntax::is_whitespace(c) ||
            c == '_' ||
            c == '\r' || c == '\n'
        {
            continue;
        }

        let digit = match c.to_digit(1 << bits_per_char)
        {
            Some(digit) => digit,
            None =>
            {
                info.report.error_span(
                    "invalid character in file contents",
                    &info.span);
                
                return Err(());
            }
        };
        
        for i in 0..bits_per_char
        {
            let bit = (digit & (1 << (bits_per_char - 1 - i))) != 0;
            bitvec.write(bitvec.len(), bit);
        }
    }

    // TODO: Optimize conversion to bigint
    Ok(expr::Value::make_integer(bitvec.as_bigint()))
}