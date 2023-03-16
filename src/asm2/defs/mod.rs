use crate::*;


mod bankdef;
pub use bankdef::{
    Bankdef,
};

mod ruledef;
pub use ruledef::{
    Ruledef,
    Rule,
    RuleParameter,
    RuleParameterType,
    RulePattern,
    RulePatternPart,
};

mod symbol;
pub use symbol::{
    Symbol,
};


#[derive(Debug)]
pub struct ItemDefs
{
    pub bankdefs: DefList<Bankdef>,
    pub ruledefs: DefList<Ruledef>,
    pub symbols: DefList<Symbol>,
}


#[derive(Debug)]
pub struct DefList<T>
{
    pub defs: Vec<T>,
}


impl<T> DefList<T>
{
    pub fn new() -> DefList<T>
    {
        DefList::<T> {
            defs: Vec::new(),
        }
    }


    pub fn define(&mut self, item_ref: util::ItemRef<T>, item: T)
    {
        self.defs.insert(item_ref.0, item);
    }


    pub fn get(&self, item_ref: util::ItemRef<T>) -> &T
    {
        &self.defs[item_ref.0]
    }


    pub fn get_mut(&mut self, item_ref: util::ItemRef<T>) -> &mut T
    {
        &mut self.defs[item_ref.0]
    }
}


pub fn resolve(
    report: &mut diagn::Report,
    ast: &asm2::parser::AstTopLevel,
    decls: &mut asm2::decls::ItemDecls)
    -> Result<ItemDefs, ()>
{
    let mut defs = ItemDefs {
        bankdefs: DefList::new(),
        ruledefs: DefList::new(),
        symbols: DefList::new(),
    };


    let guard = report.get_error_guard();

    bankdef::resolve(report, ast, decls, &mut defs)?;
    ruledef::resolve(report, ast, decls, &mut defs)?;
    symbol::resolve(report, ast, decls, &mut defs)?;

    report.stop_at_errors(guard)?;


    Ok(defs)
}