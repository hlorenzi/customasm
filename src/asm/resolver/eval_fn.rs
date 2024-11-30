use crate::*;


type BuiltinFn = fn(
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalFunctionQuery)
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


pub fn get_statically_known_builtin_fn(
    query: &expr::StaticallyKnownFunctionQuery)
    -> bool
{
    match query.func.as_ref()
    {
        "incbin" => true,
        "incbinstr" => true,
        "inchexstr" => true,
        _ => false,
    }
}


pub fn eval_fn(
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    if let expr::Value::AsmBuiltInFunction(ref name) = query.func
    {
        let builtin_fn = resolve_builtin_fn(name).unwrap();

        builtin_fn(
            fileserver,
            decls,
            defs,
            ctx,
            query)
    }
    else if let expr::Value::Function(fn_index) = query.func
    {
        query.eval_ctx.check_recursion_depth_limit(
            query.report,
            query.span)?;
        
        let fn_ref = util::ItemRef::<asm::Function>::new(fn_index);
        let function = defs.functions.get(fn_ref);
        let symbol_decl = decls.symbols.get(function.item_ref);
        
        let mut args_ctx = expr::EvalContext::new_deepened(
            &query.eval_ctx);

        query.ensure_arg_number(function.params.len())?;

        for param_index in 0..function.params.len()
        {
            let param = &function.params[param_index];
            let arg = &query.args[param_index];
            
            args_ctx.set_local(
                param.name.clone(),
                arg.value.clone());
        }

        query.report.push_parent(
            "failed to resolve function call",
            query.span);

        query.report.push_parent_short_note(
            format!(
                "within function `{}`",
                symbol_decl.name),
            symbol_decl.span);

        let maybe_result = asm::resolver::eval(
            query.report,
            opts,
            fileserver,
            decls,
            defs,
            ctx,
            &mut args_ctx,
            &function.body);

        query.report.pop_parent();
        query.report.pop_parent();

        maybe_result
    }
    else
    {
        unreachable!()
    }
}


fn eval_builtin_incbin(
    fileserver: &mut dyn util::FileServer,
    _decls: &asm::ItemDecls,
    _defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_min_max_arg_number(1, 3)?;

    let relative_filename = query.args[0].value.expect_string(
        query.report,
        query.args[0].span)?;

    let filename_ctx = fileserver.get_filename(
        ctx.file_handle_ctx.unwrap());

    let absolute_filename = util::filename_navigate(
        query.report,
        query.args[0].span,
        filename_ctx,
        &relative_filename.utf8_contents)?;

    let file_handle = fileserver.get_handle(
        query.report,
        Some(query.args[0].span),
        &absolute_filename)?;

    let bytes = fileserver.get_bytes(
        query.report,
        Some(query.args[0].span),
        file_handle)?;

    let start = {
        if query.args.len() >= 2
        {
            query.args[1].value.expect_usize(
                query.report,
                query.args[1].span)?
        }
        else
        {
            0
        }
    };

    let end = {
        if query.args.len() >= 3
        {
            let size = query.args[2].value.expect_usize(
                query.report,
                query.args[2].span)?;

            start + size
        }
        else
        {
            bytes.len()
        }
    };

    if bytes.len() == 0
    {
        return Ok(expr::Value::make_integer(util::BigInt::from_bytes_be(&[])));
    }

    if start >= bytes.len()
    {
        query.report.error_span(
            format!(
                "`incbin` range starts after EOF ({} >= {})",
                start,
                bytes.len()),
            query.args[1].span);
        return Err(());
    }

    if end > bytes.len()
    {
        query.report.error_span(
            format!(
                "`incbin` range ends after EOF ({} + {} > {})",
                start,
                end - start,
                bytes.len()),
            query.args[2].span);
        return Err(());
    }

    Ok(expr::Value::make_integer(
        util::BigInt::from_bytes_be(&bytes[start..end])))
}


fn eval_builtin_incbinstr(
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_incstr(
        fileserver,
        1,
        decls,
        defs,
        ctx,
        query,
        "incbinstr")
}


fn eval_builtin_inchexstr(
    fileserver: &mut dyn util::FileServer,
    decls: &asm::ItemDecls,
    defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    eval_builtin_incstr(
        fileserver,
        4,
        decls,
        defs,
        ctx,
        query,
        "inchexstr")
}


fn eval_builtin_incstr(
    fileserver: &mut dyn util::FileServer,
    bits_per_char: usize,
    _decls: &asm::ItemDecls,
    _defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalFunctionQuery,
    funcname: &str)
    -> Result<expr::Value, ()>
{
    query.ensure_min_max_arg_number(1, 3)?;

    let relative_filename = query.args[0].value.expect_string(
        query.report,
        query.args[0].span)?;

    let filename_ctx = fileserver.get_filename(
        ctx.file_handle_ctx.unwrap());
    
    let absolute_filename = util::filename_navigate(
        query.report,
        query.args[0].span,
        filename_ctx,
        &relative_filename.utf8_contents)?;

    let file_handle = fileserver.get_handle(
        query.report,
        Some(query.args[0].span),
        &absolute_filename)?;
    
    let contents = fileserver.get_str(
        query.report,
        Some(query.args[0].span),
        file_handle)?;

    
    let mut bitvec = util::BitVec::new();

    for c in contents.chars()
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
                    query.report.error_span(
                        "invalid character in file contents",
                        query.span);
                    
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

    let bigint = bitvec.to_bigint();

    let bigint_size = bigint.size.unwrap();

    let start = {
        if query.args.len() >= 2
        {
            query.args[1].value.expect_usize(
                query.report,
                query.args[1].span)?
        }
        else
        {
            0
        }
    };

    let end = {
        if query.args.len() >= 3
        {
            let size = query.args[2].value.expect_usize(
                query.report,
                query.args[2].span)?;

            start + size
        }
        else
        {
            bigint_size / bits_per_char
        }
    };

    if (start * bits_per_char) >= bigint_size
    {
        query.report.error_span(
            format!(
                "`{}` range starts after EOF ({} >= {})",
                funcname,
                start,
                bigint_size / bits_per_char),
            query.args[1].span);
        return Err(());
    }

    if (end * bits_per_char) > bigint_size
    {
        query.report.error_span(
            format!(
                "`{}` range ends after EOF ({} + {} > {})",
                funcname,
                start,
                end - start,
                bigint_size / bits_per_char),
            query.args[2].span);
        return Err(());
    }

    Ok(expr::Value::make_integer(
        bigint.slice(
            bigint_size - (start * bits_per_char),
            bigint_size - (end * bits_per_char))))
}
