use crate::*;


mod ruledef;
pub use ruledef::Ruledef;


#[derive(Debug)]
pub struct ItemDefs
{
    pub ruledefs: DefList<Ruledef>,
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


    pub fn set(&mut self, item_ref: asm2::decls::ItemRef<T>, item: T)
    {
        self.defs.insert(item_ref.0, item);
    }


    pub fn get(&self, item_ref: asm2::decls::ItemRef<T>) -> &T
    {
        &self.defs[item_ref.0]
    }


    pub fn get_mut(&mut self, item_ref: asm2::decls::ItemRef<T>) -> &mut T
    {
        &mut self.defs[item_ref.0]
    }
}


pub fn resolve(
    report: &mut diagn::Report,
    ast: &asm2::parser::AstTopLevel,
    decls: &asm2::decls::ItemDecls)
    -> Result<ItemDefs, ()>
{
    let mut defs = ItemDefs {
        ruledefs: DefList::new(),
    };


    let guard = report.get_error_guard();

    ruledef::resolve(report, ast, decls, &mut defs)?;

    report.stop_at_errors(guard)?;


    Ok(defs)
}