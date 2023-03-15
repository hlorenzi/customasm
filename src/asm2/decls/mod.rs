use crate::*;


mod bankdef;
mod ruledef;
mod symbol;


#[derive(Debug)]
pub struct ItemDecls
{
    pub banks: util::SymbolManager<asm2::Bankdef>,
    pub ruledefs: util::SymbolManager<asm2::Ruledef>,
    pub symbols: util::SymbolManager<asm2::Symbol>,
}


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm2::AstTopLevel)
    -> Result<ItemDecls, ()>
{
    let mut collections = ItemDecls {
        banks: util::SymbolManager::new("bank"),
        ruledefs: util::SymbolManager::new("ruledef"),
        symbols: util::SymbolManager::new("symbol"),
    };


    let guard = report.get_error_guard();

    bankdef::collect(report, ast, &mut collections)?;
    ruledef::collect(report, ast, &mut collections)?;
    symbol::collect(report, ast, &mut collections)?;

    report.stop_at_errors(guard)?;


    Ok(collections)
}