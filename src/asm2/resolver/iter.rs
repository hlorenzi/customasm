use crate::*;


pub struct AstIteratorWithContext<'a>
{
    ast: &'a asm2::AstTopLevel,
    decls: &'a asm2::ItemDecls,
    index: usize,
    symbol_ctx: Option<&'a util::SymbolContext>,
    bank_ref: util::ItemRef<asm2::Bankdef>,
}


pub struct AstIteratorWithContextItem<'a>
{
    pub node: &'a asm2::AstAny,
    pub bank_ref: util::ItemRef<asm2::Bankdef>,
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
        bank_ref: util::ItemRef::new(0),
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
        
        match ast_any
        {
            asm2::AstAny::DirectiveBank(ast_bank) =>
            {
                self.bank_ref = ast_bank.item_ref.unwrap();
            }

            asm2::AstAny::Symbol(ast_symbol) =>
            {
                let item_ref = ast_symbol.item_ref.unwrap();
                let decl = self.decls.symbols.get(item_ref);

                self.symbol_ctx = Some(&decl.ctx);
            }

            _ => {}
        }

        Some(AstIteratorWithContextItem {
            node: ast_any,
            maybe_symbol_ctx: self.symbol_ctx,
            bank_ref: self.bank_ref,
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