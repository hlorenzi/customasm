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

mod eval;
pub use eval::{
    eval,
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
    -> Result<ResolutionState, ()>
{
    for i in 0..max_iterations
    {
        let resolution_state = resolve_once(
            report,
            ast,
            decls,
            defs,
            i + 1 == max_iterations)?;

        if let asm2::ResolutionState::Resolved = resolution_state
        {
            return Ok(asm2::ResolutionState::Resolved);
        }
    }

    Ok(asm2::ResolutionState::Unresolved)
}


pub fn resolve_once(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs,
    is_final_iteration: bool)
    -> Result<asm2::ResolutionState, ()>
{
    let mut resolution_state = asm2::ResolutionState::Resolved;

    let mut iter = ResolveIterator::new(
        ast,
        defs,
        is_final_iteration);

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

        iter.update_after_node(decls, defs);
    }

    Ok(resolution_state)
}