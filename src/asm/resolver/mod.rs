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
    resolve_constants_simple_once,
    resolve_constant,
};

mod label;
mod instruction;
mod data_block;
mod res;
mod align;
mod addr;

mod eval;
pub use eval::{
    eval,
    eval_simple,
    eval_certain,
    eval_variable,
    eval_variable_simple,
    eval_variable_certain,
};

mod eval_asm;
pub use eval_asm::{
    eval_asm,
};

mod eval_fn;
pub use eval_fn::{
    resolve_builtin_fn,
    get_statically_known_builtin_fn,
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
        if let asm::ResolverNode::Symbol(ast_symbol) = ctx.node
        {
            if let asm::AstSymbolKind::Constant(_) = ast_symbol.kind
            {
                resolution_state.merge(
                    resolve_constant(
                        report,
                        opts,
                        fileserver,
                        ast_symbol,
                        decls,
                        defs,
                        &ctx)?);
            }
            else
            {
                resolution_state.merge(
                    label::resolve_label(
                        report,
                        opts,
                        ast_symbol,
                        decls,
                        defs,
                        &ctx)?);
            }
        }
        
        else if let asm::ResolverNode::Instruction(ast_instr) = ctx.node
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
        
        else if let asm::ResolverNode::DataElement(ast_data, elem_index) = ctx.node
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
        
        else if let asm::ResolverNode::Res(ast_res) = ctx.node
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
        
        else if let asm::ResolverNode::Align(ast_align) = ctx.node
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
        
        else if let asm::ResolverNode::Addr(ast_addr) = ctx.node
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
    }

    Ok(resolution_state)
}