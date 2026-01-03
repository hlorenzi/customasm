use crate::*;


mod iter;
pub use iter::{
    ResolveIterator,
    ResolverContext,
    ResolverNode,
    BankData,
};

mod constant;
pub use constant::{
    resolve_constants_simple,
    resolve_constant,
};

mod label;
mod instruction;
mod data_block;
mod res;
mod align;
mod addr;
mod assert;

mod directive_if;
pub use directive_if::{
    resolve_ifs,
    check_leftover_ifs,
};

mod eval;
pub use eval::{
    eval,
    eval_simple,
    eval_certain,
    eval_ctxlabel,
    eval_variable,
    eval_variable_simple,
    eval_variable_certain,
    eval_member,
};

mod eval_asm;
pub use eval_asm::eval_asm;

mod eval_fn;
pub use eval_fn::{
    AsmBuiltinFn,
    resolve_builtin_fn,
    get_builtin_fn_eval,
    get_statically_known_builtin_fn,
    resolve_and_get_statically_known_builtin_fn,
    eval_fn,
};


pub enum ResolutionState
{
    Unresolved,
    Resolved,
}


impl ResolutionState
{
    pub fn merge(&mut self, other: ResolutionState)
    {
        if let ResolutionState::Unresolved = other
        {
            *self = ResolutionState::Unresolved;
        }
    }
}


pub fn resolve_iteratively(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast: &asm::AstTopLevel,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    max_iterations: usize)
    -> Result<usize, ()>
{
    let mut iter_count = 0;

    while iter_count < max_iterations
    {
        iter_count += 1;

        let is_first_iteration = iter_count == 1;
        let is_last_iteration = iter_count == max_iterations;

        let resolution_state = resolve_once(
            report,
            opts,
            fileserver,
            ast,
            decls,
            defs,
            iter_count,
            is_first_iteration,
            is_last_iteration)?;

        if let asm::ResolutionState::Resolved = resolution_state
        {
            if is_last_iteration
            {
                return Ok(iter_count);
            }
            
            break;
        }
        else if is_last_iteration
        {
            return Err(());
        }
    }

    // Attempt another resolve pass
    // as if it were the last iteration
    let resolution_state = resolve_once(
        report,
        opts,
        fileserver,
        ast,
        decls,
        defs,
        iter_count + 1,
        false,
        true)?;

    if let asm::ResolutionState::Resolved = resolution_state
    {
        Ok(iter_count)
    }
    else
    {
        Err(())
    }
}


pub fn resolve_once(
    report: &mut diagn::Report,
    opts: &asm::AssemblyOptions,
    fileserver: &mut dyn util::FileServer,
    ast: &asm::AstTopLevel,
    decls: &asm::ItemDecls,
    defs: &mut asm::ItemDefs,
    iteration_index: usize,
    is_first_iteration: bool,
    is_last_iteration: bool)
    -> Result<asm::ResolutionState, ()>
{
    if opts.debug_iterations
    {
        println!(
            "[===== iteration #{} {}=====]",
            iteration_index,
            if is_last_iteration { "(final) " } else { "" });
    }

    let mut resolution_state = asm::ResolutionState::Resolved;

    let mut iter = ResolveIterator::new(
        ast,
        defs,
        is_first_iteration,
        is_last_iteration);

    while let Some(ctx) = iter.next(report, decls, defs)?
    {
        match ctx.node
        {
            asm::ResolverNode::None => {}
            
            asm::ResolverNode::Symbol(ast_symbol) =>
            {
                match ast_symbol.kind
                {
                    asm::AstSymbolKind::Constant(_) =>
                        resolution_state.merge(
                            resolve_constant(
                                report,
                                opts,
                                fileserver,
                                ast_symbol,
                                decls,
                                defs,
                                &ctx)?),

                    asm::AstSymbolKind::Label =>
                        resolution_state.merge(
                            label::resolve_label(
                                report,
                                opts,
                                ast_symbol,
                                decls,
                                defs,
                                &ctx)?),
                }
            }
        
            asm::ResolverNode::Instruction(ast_instr) =>
            {
                resolution_state.merge(
                    instruction::resolve_instruction(
                        report,
                        opts,
                        fileserver,
                        ast_instr,
                        decls,
                        defs,
                        &ctx)?);
            }
        
            asm::ResolverNode::DataElement(ast_data, elem_index) =>
            {
                resolution_state.merge(
                    data_block::resolve_data_element(
                        report,
                        opts,
                        fileserver,
                        ast_data,
                        elem_index,
                        decls,
                        defs,
                        &ctx)?);
            }
        
            asm::ResolverNode::Res(ast_res) =>
            {
                resolution_state.merge(
                    res::resolve_res(
                        report,
                        opts,
                        fileserver,
                        ast_res,
                        decls,
                        defs,
                        &ctx)?);
            }
        
            asm::ResolverNode::Align(ast_align) =>
            {
                resolution_state.merge(
                    align::resolve_align(
                        report,
                        opts,
                        fileserver,
                        ast_align,
                        decls,
                        defs,
                        &ctx)?);
            }
        
            asm::ResolverNode::Addr(ast_addr) =>
            {
                resolution_state.merge(
                    addr::resolve_addr(
                        report,
                        opts,
                        fileserver,
                        ast_addr,
                        decls,
                        defs,
                        &ctx)?);
            }
        
            asm::ResolverNode::Assert(ast_assert) =>
            {
                resolution_state.merge(
                    assert::resolve_assert(
                        report,
                        opts,
                        fileserver,
                        ast_assert,
                        decls,
                        defs,
                        &ctx)?);
            }
        }
    }

    Ok(resolution_state)
}