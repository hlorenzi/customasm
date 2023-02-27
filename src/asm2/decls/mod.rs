use crate::*;


mod ruledef;


#[derive(Debug)]
pub struct ItemDecls
{
    pub ruledefs: DeclMaps<asm2::defs::Ruledef>,
}


#[derive(Debug)]
pub struct Decl
{
    pub span: diagn::Span,
    pub name: String,
}


#[derive(Debug)]
pub struct DeclMaps<T>
{
    pub def_index: usize,
    pub decls: Vec<Decl>,
    pub name_map: std::collections::HashMap<String, ItemRef<T>>,
    pub span_refs: std::collections::HashMap<diagn::Span, ItemRef<T>>,
}


#[derive(Debug)]
pub struct ItemRef<T>(pub(super) usize, std::marker::PhantomData<*const T>);


impl<T> ItemRef<T>
{
    fn new(value: usize) -> Self
    {
        ItemRef(value, std::marker::PhantomData)
    }
}


impl<T> Clone for ItemRef<T>
{
    fn clone(&self) -> Self
    {
        ItemRef(self.0, std::marker::PhantomData)
    }
}


impl<T> Copy for ItemRef<T> {}


impl<T> DeclMaps<T>
{
    pub fn new() -> DeclMaps<T>
    {
        DeclMaps::<T> {
            def_index: 0,
            decls: Vec::new(),
            name_map: std::collections::HashMap::new(),
            span_refs: std::collections::HashMap::new(),
        }
    }


    pub fn register(&mut self, name: String, decl_span: diagn::Span) -> ItemRef<T>
    {
        let index = self.def_index;
        self.def_index += 1;

        let item_ref = ItemRef::new(index);

        self.name_map.insert(name.clone(), item_ref);
        self.decls.push(Decl {
            span: decl_span,
            name,
        });

        item_ref
    }


    pub fn add_span_ref(&mut self, span: diagn::Span, item_ref: ItemRef<T>)
    {
        self.span_refs.insert(span, item_ref);
    }


    pub fn get_from_name(&self, name: &str) -> Option<ItemRef<T>>
    {
        self.name_map.get(name).copied()
    }


    pub fn get_from_span(&self, span: &diagn::Span) -> Option<ItemRef<T>>
    {
        self.span_refs.get(span).copied()
    }


    pub fn get(&self, item_ref: ItemRef<T>) -> &Decl
    {
        &self.decls[item_ref.0]
    }
}


pub fn collect(
    report: &mut diagn::Report,
    ast: &mut asm2::parser::AstTopLevel)
    -> Result<ItemDecls, ()>
{
    let mut collections = ItemDecls {
        ruledefs: DeclMaps::new(),
    };


    let guard = report.get_error_guard();

    ruledef::collect(report, ast, &mut collections)?;

    report.stop_at_errors(guard)?;


    Ok(collections)
}