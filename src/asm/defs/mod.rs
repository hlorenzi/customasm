use crate::*;


mod bankdef;
pub use bankdef::Bankdef;

mod ruledef;
pub use ruledef::{
    Ruledef,
    Rule,
    RuleParameter,
    RuleParameterType,
    RulePattern,
    RulePatternPart,
};

mod ruledef_map;
pub use ruledef_map::{
    RuledefMap,
    RuledefMapEntry,
};

mod symbol;
pub use symbol::Symbol;

mod function;
pub use function::{
    Function,
    FunctionParameter,
};

mod instruction;
pub use instruction::Instruction;

mod data_block;
pub use data_block::DataElement;

mod res;
pub use res::ResDirective;

mod align;
pub use align::AlignDirective;

mod addr;
pub use addr::AddrDirective;


#[derive(Debug)]
pub struct ItemDefs
{
    pub symbols: DefList<Symbol>,
    pub bankdefs: DefList<Bankdef>,
    pub ruledefs: DefList<Ruledef>,
    pub ruledef_map: RuledefMap,
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
    pub defs: Vec<Option<T>>,
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
        while item_ref.0 >= self.defs.len()
        {
            self.defs.push(None);
        }

        assert!(self.defs[item_ref.0].is_none());

        self.defs[item_ref.0] = Some(item);
    }


    pub fn len(&self) -> usize
    {
        self.defs.len()
    }


    pub fn get(&self, item_ref: util::ItemRef<T>) -> &T
    {
        self.defs[item_ref.0].as_ref().unwrap()
    }


    pub fn get_mut(&mut self, item_ref: util::ItemRef<T>) -> &mut T
    {
        self.defs[item_ref.0].as_mut().unwrap()
    }


    pub fn maybe_get(&self, item_ref: util::ItemRef<T>) -> Option<&T>
    {
        if item_ref.0 >= self.defs.len()
        {
            None
        }
        else
        {
            Some(self.defs[item_ref.0].as_ref().unwrap())
        }
    }


    pub fn next_item_ref(&self) -> util::ItemRef<T>
    {
        util::ItemRef::new(self.defs.len())
    }
}


pub fn init() -> ItemDefs
{
    ItemDefs {
        symbols: DefList::new(),
        bankdefs: DefList::new(),
        ruledefs: DefList::new(),
        ruledef_map: RuledefMap::new(),
        functions: DefList::new(),
        instructions: DefList::new(),
        data_elems: DefList::new(),
        res_directives: DefList::new(),
        align_directives: DefList::new(),
        addr_directives: DefList::new(),
    }
}


pub fn define_symbols(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast: &mut asm::parser::AstTopLevel,
    decls: &asm::decls::ItemDecls,
    defs: &mut asm::defs::ItemDefs)
    -> Result<(), ()>
{
    symbol::define(report, opts, ast, decls, defs)?;
    
    report.stop_at_errors()?;

    Ok(())
}


pub fn define_remaining(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    ast: &mut asm::parser::AstTopLevel,
    defs: &mut asm::defs::ItemDefs,
    decls: &mut asm::decls::ItemDecls)
    -> Result<(), ()>
{
    bankdef::define(report, opts, ast, decls, defs)?;
    ruledef::define(report, ast, decls, defs)?;
    function::define(report, ast, decls, defs)?;
    instruction::define(report, ast, decls, defs)?;
    data_block::define(report, opts, ast, decls, defs)?;
    res::define(report, ast, decls, defs)?;
    align::define(report, ast, decls, defs)?;
    addr::define(report, ast, decls, defs)?;
    
    report.stop_at_errors()?;

    if opts.optimize_instruction_matching
    {
        defs.ruledef_map.build(&defs.ruledefs);
    }

    Ok(())
}