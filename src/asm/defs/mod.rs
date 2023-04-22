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

mod function;
pub use function::{
    Function,
    FunctionParameter,
};

mod instruction;
pub use instruction::{
    Instruction,
};

mod data_block;
pub use data_block::{
    DataElement,
};

mod res;
pub use res::{
    ResDirective,
};

mod align;
pub use align::{
    AlignDirective,
};

mod addr;
pub use addr::{
    AddrDirective,
};


#[derive(Debug)]
pub struct ItemDefs
{
    pub bankdefs: DefList<Bankdef>,
    pub ruledefs: DefList<Ruledef>,
    pub symbols: DefList<Symbol>,
    pub functions: DefList<Function>,
    pub instructions: DefList<Instruction>,
    pub data_elems: DefList<DataElement>,
    pub res_directives: DefList<ResDirective>,
    pub align_directives: DefList<AlignDirective>,
    pub addr_directives: DefList<AddrDirective>,
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


    pub fn len(&self) -> usize
    {
        self.defs.len()
    }


    pub fn get(&self, item_ref: util::ItemRef<T>) -> &T
    {
        &self.defs[item_ref.0]
    }


    pub fn get_mut(&mut self, item_ref: util::ItemRef<T>) -> &mut T
    {
        &mut self.defs[item_ref.0]
    }


    pub fn next_item_ref(&self) -> util::ItemRef<T>
    {
        util::ItemRef::new(self.defs.len())
    }
}


pub fn define(
    report: &mut diagn::Report,
    ast: &mut asm::parser::AstTopLevel,
    decls: &mut asm::decls::ItemDecls)
    -> Result<ItemDefs, ()>
{
    let mut defs = ItemDefs {
        bankdefs: DefList::new(),
        ruledefs: DefList::new(),
        symbols: DefList::new(),
        functions: DefList::new(),
        instructions: DefList::new(),
        data_elems: DefList::new(),
        res_directives: DefList::new(),
        align_directives: DefList::new(),
        addr_directives: DefList::new(),
    };


    bankdef::define(report, ast, decls, &mut defs)?;
    ruledef::define(report, ast, decls, &mut defs)?;
    symbol::define(report, ast, decls, &mut defs)?;
    function::define(report, ast, decls, &mut defs)?;
    instruction::define(report, ast, decls, &mut defs)?;
    data_block::define(report, ast, decls, &mut defs)?;
    res::define(report, ast, decls, &mut defs)?;
    align::define(report, ast, decls, &mut defs)?;
    addr::define(report, ast, decls, &mut defs)?;

    report.stop_at_errors()?;


    Ok(defs)
}