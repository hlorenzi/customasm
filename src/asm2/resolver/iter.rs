use crate::*;


pub struct AstIteratorWithContext<'a>
{
    ast: &'a asm2::AstTopLevel,
    decls: &'a asm2::ItemDecls,
    index: usize,
    symbol_ctx: Option<&'a util::SymbolContext>,
}


pub struct AstIteratorWithContextItem<'a>
{
    pub node: &'a asm2::AstAny,
    pub maybe_symbol_ctx: Option<&'a util::SymbolContext>,
}


pub fn iter_with_context<'a>(
    ast: &'a asm2::AstTopLevel,
    decls: &'a asm2::ItemDecls)
    -> AstIteratorWithContext<'a>
{
    AstIteratorWithContext {
        ast,
        decls,
        index: 0,
        symbol_ctx: None,
    }
}


impl<'a> Iterator for AstIteratorWithContext<'a>
{
    type Item = AstIteratorWithContextItem<'a>;


    fn next(&mut self) -> Option<Self::Item>
    {
        if self.index >= self.ast.nodes.len()
        {
            return None;
        }

        let ast_any = &self.ast.nodes[self.index];
        self.index += 1;
        
        if let asm2::AstAny::Symbol(ast_symbol) = ast_any
        {
            let item_ref = ast_symbol.item_ref.unwrap();
            let decl = self.decls.symbols.get(item_ref);

            self.symbol_ctx = Some(&decl.ctx);
        }

        Some(AstIteratorWithContextItem {
            node: ast_any,
            maybe_symbol_ctx: self.symbol_ctx,
        })
    }
}


impl<'a> AstIteratorWithContextItem<'a>
{
    pub fn get_symbol_ctx(&self) -> util::SymbolContext
    {
        match self.maybe_symbol_ctx
        {
            None => util::SymbolContext::new_global(),
            Some(ctx) => ctx.clone(),
        }
    }
}