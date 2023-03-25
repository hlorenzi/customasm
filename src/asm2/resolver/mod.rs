use crate::*;


mod iter;
pub use iter::{
    ResolveIterator,
    ResolverContext,
};

mod constant;
pub use constant::{
    resolve_constants,
    resolve_constants_once,
    resolve_constant,
};

mod label;
pub use label::{
    resolve_label,
};

mod instruction;
pub use instruction::{
    resolve_instruction,
};

mod data_block;
pub use data_block::{
    resolve_data_block,
};

mod res;
pub use res::{
    resolve_res,
};

mod eval;
pub use eval::{
    eval,
    eval_simple,
    get_current_address,
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
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
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
            ast,
            decls,
            defs,
            is_first_iteration,
            is_last_iteration)?;

        if let asm2::ResolutionState::Resolved = resolution_state
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
        ast,
        decls,
        defs,
        false,
        true)?;

    if let asm2::ResolutionState::Resolved = resolution_state
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
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    is_first_iteration: bool,
    is_last_iteration: bool)
    -> Result<asm2::ResolutionState, ()>
{
    println!(
        "=== resolve_once {}===",
        if is_last_iteration { "(final) " } else { "" });

    let mut resolution_state = asm2::ResolutionState::Resolved;

    let mut iter = ResolveIterator::new(
        ast,
        defs,
        is_first_iteration,
        is_last_iteration);

    while let Some(ctx) = iter.next(decls, defs)
    {
        if let asm2::AstAny::Symbol(ast_symbol) = ctx.node
        {
            if let asm2::AstSymbolKind::Constant(_) = ast_symbol.kind
            {
                resolution_state.merge(
                    resolve_constant(
                        report,
                        ast_symbol,
                        decls,
                        defs,
                        &ctx)?);
            }
            else
            {
                resolution_state.merge(
                    resolve_label(
                        report,
                        ast_symbol,
                        decls,
                        defs,
                        &ctx)?);
            }
        }
        
        else if let asm2::AstAny::Instruction(ast_instr) = ctx.node
        {
            resolution_state.merge(
                resolve_instruction(
                    report,
                    ast_instr,
                    decls,
                    defs,
                    &ctx)?);
        }
        
        else if let asm2::AstAny::DirectiveData(ast_data) = ctx.node
        {
            resolution_state.merge(
                resolve_data_block(
                    report,
                    ast_data,
                    decls,
                    defs,
                    &ctx)?);
        }
        
        else if let asm2::AstAny::DirectiveRes(ast_res) = ctx.node
        {
            resolution_state.merge(
                resolve_res(
                    report,
                    ast_res,
                    decls,
                    defs,
                    &ctx)?);
        }

        iter.update_after_node(decls, defs);
    }

    Ok(resolution_state)
}