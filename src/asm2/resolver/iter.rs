use crate::*;


pub struct ResolveIterator<'ast, 'decls>
{
    ast: &'ast asm2::AstTopLevel,

    /// Index into the AST
    index: usize,

    /// Index into subelements of an AST node,
    /// like the individual elements of a data directive
    subindex: usize,

    /// Value of `index` on the previous iteration
    index_prev: Option<usize>,

    /// Value of `subindex` on the previous iteration
    subindex_prev: Option<usize>,

    is_first_iteration: bool,
    is_last_iteration: bool,
    symbol_ctx: &'decls util::SymbolContext,
    bank_ref: util::ItemRef<asm2::Bankdef>,
    bank_data: Vec<BankData>,
}


#[derive(Copy, Clone, Debug)]
pub struct BankData
{
    pub cur_position: usize,
}


pub enum ResolverNode<'ast>
{
    None,
    Symbol(&'ast asm2::AstSymbol),
    Instruction(&'ast asm2::AstInstruction),
    DataElement(&'ast asm2::AstDirectiveData, usize),
    Res(&'ast asm2::AstDirectiveRes),
}


pub struct ResolverContext<'iter, 'ast, 'decls>
{
    pub node: ResolverNode<'ast>,
    pub is_first_iteration: bool,
    pub is_last_iteration: bool,
    pub symbol_ctx: &'decls util::SymbolContext,
    pub bank_ref: util::ItemRef<asm2::Bankdef>,
    pub bank_data: &'iter BankData,
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
            cur_position: 0,
        };
    
        let bank_data = vec![bank_datum; defs.bankdefs.len()];
    
        static GLOBAL_SYMBOL_CTX: util::SymbolContext =
            util::SymbolContext::new_global();

        ResolveIterator {
            ast,
            index: 0,
            subindex: 0,
            index_prev: None,
            subindex_prev: None,
            is_first_iteration,
            is_last_iteration,
            symbol_ctx: &GLOBAL_SYMBOL_CTX,
            bank_ref: util::ItemRef::new(0),
            bank_data,
        }
    }


    pub fn next<'iter>(
        &'iter mut self,
        decls: &'decls asm2::ItemDecls,
        defs: &asm2::ItemDefs)
        -> Option<ResolverContext<'iter, 'ast, 'decls>>
    {
        self.advance_address(decls, defs);

        if self.index >= self.ast.nodes.len()
        {
            return None;
        }

        self.index_prev = Some(self.index);
        self.subindex_prev = Some(self.subindex);

        let ast_any = &self.ast.nodes[self.index];

        let node = {
            match ast_any
            {
                asm2::AstAny::DirectiveBank(ast_bank) =>
                {
                    self.bank_ref = ast_bank.item_ref.unwrap();

                    self.index += 1;
                    ResolverNode::None
                }

                asm2::AstAny::DirectiveBankdef(ast_bankdef) =>
                {
                    self.bank_ref = ast_bankdef.item_ref.unwrap();

                    self.index += 1;
                    ResolverNode::None
                }

                asm2::AstAny::Symbol(ast_symbol) =>
                {
                    let item_ref = ast_symbol.item_ref.unwrap();
                    let decl = decls.symbols.get(item_ref);

                    self.symbol_ctx = &decl.ctx;

                    self.index += 1;
                    ResolverNode::Symbol(ast_symbol)
                }

                asm2::AstAny::Instruction(ast_instr) =>
                {
                    self.index += 1;
                    ResolverNode::Instruction(ast_instr)
                }

                asm2::AstAny::DirectiveData(ast_data) =>
                {
                    let elem_index = self.subindex;

                    self.subindex += 1;
                    if self.subindex >= ast_data.item_refs.len()
                    {
                        self.index += 1;
                        self.subindex = 0;
                    }

                    ResolverNode::DataElement(ast_data, elem_index)
                }

                _ =>
                {
                    self.index += 1;
                    ResolverNode::None
                }
            }
        };

        Some(ResolverContext {
            node,
            is_first_iteration: self.is_first_iteration,
            is_last_iteration: self.is_last_iteration,
            symbol_ctx: self.symbol_ctx,
            bank_ref: self.bank_ref,
            bank_data: &self.bank_data[self.bank_ref.0],
        })
    }


    fn advance_address(
        &mut self,
        decls: &'decls asm2::ItemDecls,
        defs: &asm2::ItemDefs)
    {
        if self.index_prev.is_none() ||
            self.subindex_prev.is_none()
        {
            return;
        }

        let ast_any = &self.ast.nodes[self.index_prev.unwrap()];
        
        match ast_any
        {
            asm2::AstAny::Instruction(ast_instr) =>
            {
                let item_ref = ast_instr.item_ref.unwrap();
                let instr = defs.instructions.get(item_ref);

                let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                // Advance the current bank's position
                cur_bank_data.cur_position += {
                    match instr.encoding.size
                    {
                        Some(size) => size,
                        None => 0,
                    }
                };
            }

            asm2::AstAny::DirectiveData(ast_data) =>
            {
                let item_ref = ast_data.item_refs[self.subindex_prev.unwrap()];
                let data_elem = defs.data_elems.get(item_ref);

                let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                // Advance the current bank's position
                cur_bank_data.cur_position += {
                    match data_elem.encoding.size
                    {
                        Some(size) => size,
                        None => 0,
                    }
                };
            }

            asm2::AstAny::DirectiveRes(ast_res) =>
            {
                let item_ref = ast_res.item_ref.unwrap();
                let res = defs.res_directives.get(item_ref);

                let bank = defs.bankdefs.get(self.bank_ref);
                let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                // Advance the current bank's position
                cur_bank_data.cur_position +=
                    res.reserve_size *
                    bank.addr_unit;
            }

            _ => {}
        }
    }
}


impl<'iter, 'ast, 'decls> ResolverContext<'iter, 'ast, 'decls>
{
    pub fn can_guess(&self) -> bool
    {
        !self.is_last_iteration
    }


    pub fn get_output_position(
        &self,
        defs: &asm2::ItemDefs)
        -> Option<usize>
    {
        let bank = defs.bankdefs.get(self.bank_ref);

        Some(bank.output_offset? + self.bank_data.cur_position)
    }


    pub fn get_address(
        &self,
        defs: &asm2::ItemDefs,
        can_guess: bool)
        -> Option<util::BigInt>
    {
        let bankdef = &defs.bankdefs.get(self.bank_ref);
        let addr_unit = bankdef.addr_unit;

        let cur_position = self.bank_data.cur_position;
        
        let excess_bits = cur_position % addr_unit;
        if excess_bits != 0 && !can_guess
        {
            return None;
        }
            
        let addr =
            &util::BigInt::from(cur_position / addr_unit) +
            &bankdef.addr_start;
        
        Some(addr)
    }


    pub fn eval_address(
        &self,
        report: &mut diagn::Report,
        span: &diagn::Span,
        defs: &asm2::ItemDefs,
        can_guess: bool)
        -> Result<util::BigInt, ()>
    {
        let bankdef = &defs.bankdefs.get(self.bank_ref);
        let addr_unit = bankdef.addr_unit;

        let cur_position = self.bank_data.cur_position;
        
        let excess_bits = cur_position % addr_unit;
        if excess_bits != 0 && !can_guess
        {
            let bits_short = addr_unit - excess_bits;

            let plural = {
                if bits_short > 1
                    { "bits" }
                else
                    { "bit" }
            };

            report.push_parent(
                "position is not aligned to an address",
                span);

            report.note(
                format!(
                    "needs {} more {} for next alignment",
                    bits_short, plural));

            report.pop_parent();

            return Err(());
        }
        
        let addr =
            &util::BigInt::from(cur_position / addr_unit) +
            &bankdef.addr_start;

        Ok(addr)
    }
}