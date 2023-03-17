use crate::*;


pub fn resolve_constants(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<(), ()>
{
    loop
    {
        let had_work = resolve_constants_once(
            report,
            ast,
            decls,
            defs)?;

        if !had_work
        {
            return Ok(());
        }
    }
}


pub fn resolve_constants_once(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<bool, ()>
{
    println!("== resolve_constants_once ==");
    let mut had_work = false;

    for item in asm2::resolver::iter_with_context(ast, decls)
    {
        if let asm2::AstAny::Symbol(ast_symbol) = item.node
        {
            had_work |= resolve_constant(
                report,
                ast_symbol,
                decls,
                defs,
                &item.get_symbol_ctx())?;
        }        
    }

    Ok(had_work)
}


pub fn resolve_constant<'symbol_ctx>(
    report: &mut diagn::Report,
    ast_symbol: &asm2::AstSymbol,
    decls: &'symbol_ctx asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    symbol_ctx: &'symbol_ctx util::SymbolContext)
    -> Result<bool, ()>
{
    let item_ref = ast_symbol.item_ref.unwrap();

    if let asm2::AstSymbolKind::Constant(ref ast_const) = ast_symbol.kind
    {
        let symbol = defs.symbols.get(item_ref);

        if let expr::Value::Unknown = symbol.value
        {
            let value = asm2::resolver::eval(
                report,
                decls,
                defs,
                symbol_ctx,
                &ast_const.expr)?;

            println!("{} = {:?}", decls.symbols.get(item_ref).name, value);
            
            if !value.is_unknown()
            {
                let symbol = defs.symbols.get_mut(item_ref);
                symbol.value = value;

                return Ok(true);
            }
        }
    }

    Ok(false)
}