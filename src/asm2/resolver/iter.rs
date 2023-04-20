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


#[derive(Clone)]
pub enum ResolverNode<'ast>
{
    None,
    Symbol(&'ast asm2::AstSymbol),
    Instruction(&'ast asm2::AstInstruction),
    DataElement(&'ast asm2::AstDirectiveData, usize),
    Res(&'ast asm2::AstDirectiveRes),
    Align(&'ast asm2::AstDirectiveAlign),
    Addr(&'ast asm2::AstDirectiveAddr),
}


#[derive(Clone)]
pub struct ResolverContext<'iter, 'ast, 'decls>
{
    pub node: ResolverNode<'ast>,
    pub is_first_iteration: bool,
    pub is_last_iteration: bool,
    pub filename_ctx: Option<&'ast str>,
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
        self.advance_address(defs);

        if self.index >= self.ast.nodes.len()
        {
            return None;
        }

        self.index_prev = Some(self.index);
        self.subindex_prev = Some(self.subindex);

        let ast_any = &self.ast.nodes[self.index];

        let node: ResolverNode;
        let filename_ctx: Option<&str>;

        match ast_any
        {
            asm2::AstAny::DirectiveBank(ast_bank) =>
            {
                self.bank_ref = ast_bank.item_ref.unwrap();

                self.index += 1;
                node = ResolverNode::None;
                filename_ctx = Some(&ast_bank.header_span.file);
            }

            asm2::AstAny::DirectiveBankdef(ast_bankdef) =>
            {
                self.bank_ref = ast_bankdef.item_ref.unwrap();

                self.index += 1;
                node = ResolverNode::None;
                filename_ctx = Some(&ast_bankdef.header_span.file);
            }

            asm2::AstAny::Symbol(ast_symbol) =>
            {
                let item_ref = ast_symbol.item_ref.unwrap();
                let decl = decls.symbols.get(item_ref);

                self.symbol_ctx = &decl.ctx;

                // Honor `labelalign`
                let bankdef = defs.bankdefs.get(self.bank_ref);
                if let Some(label_align) = bankdef.label_align
                {
                    if decl.depth == 0
                    {
                        let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                        cur_bank_data.cur_position += bits_until_alignment(
                            cur_bank_data.cur_position,
                            label_align);
                    }
                }

                self.index += 1;
                node = ResolverNode::Symbol(ast_symbol);
                filename_ctx = Some(&ast_symbol.decl_span.file);
            }

            asm2::AstAny::Instruction(ast_instr) =>
            {
                self.index += 1;
                node = ResolverNode::Instruction(ast_instr);
                filename_ctx = Some(&ast_instr.span.file);
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

                node = ResolverNode::DataElement(ast_data, elem_index);
                filename_ctx = Some(&ast_data.header_span.file);
            }

            asm2::AstAny::DirectiveRes(ast_res) =>
            {
                self.index += 1;
                node = ResolverNode::Res(ast_res);
                filename_ctx = Some(&ast_res.header_span.file);
            }

            asm2::AstAny::DirectiveAlign(ast_align) =>
            {
                self.index += 1;
                node = ResolverNode::Align(ast_align);
                filename_ctx = Some(&ast_align.header_span.file);
            }

            asm2::AstAny::DirectiveAddr(ast_addr) =>
            {
                self.index += 1;
                node = ResolverNode::Addr(ast_addr);
                filename_ctx = Some(&ast_addr.header_span.file);
            }

            _ =>
            {
                self.index += 1;
                node = ResolverNode::None;
                filename_ctx = None;
            }
        }

        Some(ResolverContext {
            node,
            is_first_iteration: self.is_first_iteration,
            is_last_iteration: self.is_last_iteration,
            filename_ctx,
            symbol_ctx: self.symbol_ctx,
            bank_ref: self.bank_ref,
            bank_data: &self.bank_data[self.bank_ref.0],
        })
    }


    fn advance_address(
        &mut self,
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

                let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                // Advance the current bank's position
                cur_bank_data.cur_position += res.reserve_size;
            }

            asm2::AstAny::DirectiveAlign(ast_align) =>
            {
                let item_ref = ast_align.item_ref.unwrap();
                let align = defs.align_directives.get(item_ref);

                let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                cur_bank_data.cur_position += bits_until_alignment(
                    cur_bank_data.cur_position,
                    align.align_size);
            }

            asm2::AstAny::DirectiveAddr(ast_addr) =>
            {
                let item_ref = ast_addr.item_ref.unwrap();
                let addr = defs.addr_directives.get(item_ref);

                let bank = defs.bankdefs.get(self.bank_ref);
                let mut cur_bank_data = &mut self.bank_data[self.bank_ref.0];

                let new_position = {
                    if addr.address >= bank.addr_start
                    {
                        (&addr.address - &bank.addr_start)
                            .checked_to_usize()
                            .unwrap_or(0)
                            * bank.addr_unit
                    }
                    else
                    {
                        0
                    }
                };

                cur_bank_data.cur_position = new_position;
            }

            _ => {}
        }
    }
}


fn bits_until_alignment(
    position: usize,
    alignment: usize)
    -> usize
{
    if alignment == 0
    {
        return 0;
    }

    let excess_bits = position % alignment;
        
    if excess_bits != 0
    {
        alignment - excess_bits
    }
    else
    {
        0
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