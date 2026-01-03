use crate::*;


mod bank;
mod bankdef;
mod ruledef;
mod symbol;
mod function;


#[derive(Debug)]
pub struct ItemDecls
{
    pub bankdefs: util::SymbolManager<asm::Bankdef>,
    pub ruledefs: util::SymbolManager<asm::Ruledef>,
    pub symbols: util::SymbolManager<asm::Symbol>,
}


pub fn init(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions)
    -> Result<ItemDecls, ()>
{
    let mut decls = ItemDecls {
        bankdefs: util::SymbolManager::new("bank"),
        ruledefs: util::SymbolManager::new("ruledef"),
        symbols: util::SymbolManager::new("symbol"),
    };

    let initial_item_ref = decls.bankdefs.declare(
        report,
        diagn::Span::new_dummy(),
        opts,
        &util::SymbolContext::new_global(),
        None,
        "#global_bankdef".to_string(),
        0,
        util::SymbolKind::Other)?;
    
    debug_assert!(initial_item_ref.0 == 0);

    Ok(decls)
}


pub fn collect(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast: &mut asm::AstTopLevel,
    decls: &mut asm::ItemDecls)
    -> Result<(), ()>
{
    bankdef::collect(report, opts, ast, decls)?;
    bank::collect(report, ast, decls)?;
    ruledef::collect(report, opts, ast, decls)?;
    symbol::collect(report, opts, ast, decls)?;
    function::collect(report, opts, ast, decls)?;

    report.stop_at_errors()?;

    Ok(())
}