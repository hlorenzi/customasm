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
    query.ensure_arg_number(1)?;

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

    Ok(expr::Value::make_integer(
        util::BigInt::from_bytes_be(&bytes)))
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
        query)
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
        query)
}


fn eval_builtin_incstr(
    fileserver: &mut dyn util::FileServer,
    bits_per_char: usize,
    _decls: &asm::ItemDecls,
    _defs: &asm::ItemDefs,
    ctx: &asm::ResolverContext,
    query: &mut expr::EvalFunctionQuery)
    -> Result<expr::Value, ()>
{
    query.ensure_arg_number(1)?;

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
    
    let chars = fileserver.get_chars(
        query.report,
        Some(query.args[0].span),
        file_handle)?;

    
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

    Ok(expr::Value::make_integer(bitvec.to_bigint()))
}