use crate::*;


type BuiltinFn = fn(
    fileserver: &dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    info: &mut expr::EvalFunctionInfo)
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


pub fn eval_fn(
    fileserver: &dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    info: &mut expr::EvalFunctionInfo)
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
    else if let expr::Value::Function(fn_index) = info.func
    {
        info.eval_ctx.check_recursion_depth_limit(
            info.report,
            info.span)?;
        
        let fn_ref = util::ItemRef::<asm::Function>::new(fn_index);
        let function = defs.functions.get(fn_ref);
        let symbol_decl = decls.symbols.get(function.item_ref);
        
        let mut args_ctx = expr::EvalContext::new_deepened(
            &info.eval_ctx);

        info.ensure_arg_number(function.params.len())?;

        for param_index in 0..function.params.len()
        {
            let param = &function.params[param_index];
            let arg = &info.args[param_index];
            
            args_ctx.set_local(
                param.name.clone(),
                arg.value.clone());
        }

        info.report.push_parent(
            "failed to resolve function call",
            info.span);

        info.report.push_parent_short_note(
            format!(
                "within function `{}`",
                symbol_decl.name),
            &symbol_decl.span);

        let maybe_result = asm::resolver::eval(
            info.report,
            fileserver,
            decls,
            defs,
            ctx,
            &mut args_ctx,
            &function.body);

        info.report.pop_parent();
        info.report.pop_parent();

        maybe_result
    }
    else
    {
        unreachable!()
    }
}


fn eval_builtin_incbin(
    fileserver: &dyn util::FileServer,
    _decls: &asm::ItemDecls,
    _defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    info: &mut expr::EvalFunctionInfo)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let relative_filename = info.args[0].value.expect_string(
        info.report,
        info.args[0].span)?;

    let absolute_filename = util::filename_navigate(
        info.report,
        info.args[0].span,
        &ctx.filename_ctx.unwrap(),
        &relative_filename.utf8_contents)?;

    let bytes = fileserver.get_bytes(
        info.report,
        Some(info.args[0].span),
        &absolute_filename)?;

    Ok(expr::Value::make_integer(
        util::BigInt::from_bytes_be(&bytes)))
}


fn eval_builtin_incbinstr(
    fileserver: &dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    info: &mut expr::EvalFunctionInfo)
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
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    info: &mut expr::EvalFunctionInfo)
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
    _decls: &asm::ItemDecls,
    _defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    info: &mut expr::EvalFunctionInfo)
    -> Result<expr::Value, ()>
{
    info.ensure_arg_number(1)?;

    let relative_filename = info.args[0].value.expect_string(
        info.report,
        info.args[0].span)?;

    let absolute_filename = util::filename_navigate(
        info.report,
        info.args[0].span,
        &ctx.filename_ctx.unwrap(),
        &relative_filename.utf8_contents)?;

    let chars = fileserver.get_chars(
        info.report,
        Some(info.args[0].span),
        &absolute_filename)?;

    
    let mut bitvec = util::BitVec::new();

    for c in chars
    {
        if syntax::is_whitespace(c) ||
            c == '_' ||
            c == '\r' ||
            c == '\n'
        {
            continue;
        }

        let digit = {
            match c.to_digit(1 << bits_per_char)
            {
                Some(digit) => digit,
                None =>
                {
                    info.report.error_span(
                        "invalid character in file contents",
                        &info.span);
                    
                    return Err(());
                }
            }
        };
        
        for i in 0..bits_per_char
        {
            let bit = (digit & (1 << (bits_per_char - 1 - i))) != 0;
            bitvec.write_bit(bitvec.len(), bit);
        }
    }

    Ok(expr::Value::make_integer(bitvec.to_bigint()))
}