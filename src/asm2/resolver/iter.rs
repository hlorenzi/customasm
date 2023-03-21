use crate::*;


pub struct ResolveIterator<'ast, 'decls>
{
    ast: &'ast asm2::AstTopLevel,
    index: usize,
    is_first_iteration: bool,
    is_last_iteration: bool,
    symbol_ctx: &'decls util::SymbolContext,
    bank_ref: util::ItemRef<asm2::Bankdef>,
    bank_data: Vec<BankData>,
}


#[derive(Copy, Clone, Debug)]
pub struct BankData
{
    pub cur_position: Option<usize>,
    pub cur_position_guess: Option<usize>,
}


pub struct ResolverContext<'ast, 'decls>
{
    pub node: &'ast asm2::AstAny,
    pub is_first_iteration: bool,
    pub is_last_iteration: bool,
    pub symbol_ctx: &'decls util::SymbolContext,
    pub bank_ref: util::ItemRef<asm2::Bankdef>,
    pub bank_data: BankData,
}


impl<'ast, 'decls> ResolveIterator<'ast, 'decls>
{
    pub fn new<'defs>(
        ast: &'ast asm2::AstTopLevel,
        defs: &'defs asm2::ItemDefs,
        is_first_iteration: bool,
        is_last_iteration: bool)
        -> ResolveIterator<'ast, 'decls>
    {
        let bank_datum = BankData {
            cur_position: Some(0),
            cur_position_guess: Some(0),
        };
    
        let bank_data = vec![bank_datum; defs.bankdefs.len()];
    
        static GLOBAL_SYMBOL_CTX: util::SymbolContext =
            util::SymbolContext::new_global();

        ResolveIterator {
            ast,
            index: 0,
            is_first_iteration,
            is_last_iteration,
            symbol_ctx: &GLOBAL_SYMBOL_CTX,
            bank_ref: util::ItemRef::new(0),
            bank_data,
        }
    }


    pub fn next(
        &mut self,
        decls: &'decls asm2::ItemDecls,
        defs: &asm2::ItemDefs)
        -> Option<ResolverContext<'ast, 'decls>>
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

            asm2::AstAny::DirectiveBankdef(ast_bankdef) =>
            {
                self.bank_ref = ast_bankdef.item_ref.unwrap();
            }

            asm2::AstAny::Symbol(ast_symbol) =>
            {
                let item_ref = ast_symbol.item_ref.unwrap();
                let decl = decls.symbols.get(item_ref);

                self.symbol_ctx = &decl.ctx;
            }

            _ => {}
        }

        Some(ResolverContext {
            node: ast_any,
            is_first_iteration: self.is_first_iteration,
            is_last_iteration: self.is_last_iteration,
            symbol_ctx: self.symbol_ctx,
            bank_ref: self.bank_ref,
            bank_data: self.bank_data[self.bank_ref.0],
        })
    }


    pub fn update_after_node(
        &mut self,
        decls: &'decls asm2::ItemDecls,
        defs: &asm2::ItemDefs)
    {
        debug_assert!(self.index >= 1);
        debug_assert!(self.index - 1 < self.ast.nodes.len());

        let ast_any = &self.ast.nodes[self.index - 1];
        
        match ast_any
        {
            asm2::AstAny::Instruction(ast_instr) =>
            {
                let item_ref = ast_instr.item_ref.unwrap();
                let instr = defs.instructions.get(item_ref);

                let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                // Advance the current bank's position
                if let Some(ref mut addr) = cur_bank_data.cur_position
                {
                    match instr.encoding_size
                    {
                        Some(size) => *addr += size,
                        None => cur_bank_data.cur_position = None,
                    }
                }

                // Advance the current bank's position guess
                if let Some(ref mut addr) = cur_bank_data.cur_position_guess
                {
                    match instr.encoding_size
                    {
                        Some(size) => *addr += size,
                        None =>
                        {
                            match instr.encoding_size_guess
                            {
                                Some(size) => *addr += size,
                                None => {}
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }
}