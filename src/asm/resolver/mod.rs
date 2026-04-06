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
pub use instruction::{
    finalize_instruction,
};

mod data_block;
pub use data_block::{
    check_final_data_element,
};

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
    resolve_builtin_symbol,
};

mod eval_asm;
pub use eval_asm::eval_asm;

mod eval_fn;
pub use eval_fn::{
    AsmBuiltinFn,
    resolve_builtin_fn,
    get_builtin_fn_eval,
    eval_fn,
};


#[derive(Copy, Clone)]
pub enum ResolutionState
{
    Unresolved,
    Stable,
    Resolved,
}


impl ResolutionState
{
    pub fn is_resolved(&self) -> bool
    {
        match self
        {
            ResolutionState::Resolved => true,
            ResolutionState::Stable | ResolutionState::Unresolved => false,
        }
    }


    pub fn is_stable_or_resolved(&self) -> bool
    {
        match self
        {
            ResolutionState::Resolved | ResolutionState::Stable => true,
            ResolutionState::Unresolved => false,
        }
    }


    pub fn merge(&mut self, other: ResolutionState)
    {
        *self = {
            match (&self, other)
            {
                (ResolutionState::Unresolved, _) |
                (_, ResolutionState::Unresolved) =>
                    ResolutionState::Unresolved,
                
                (ResolutionState::Stable, ResolutionState::Stable) |
                (ResolutionState::Stable, ResolutionState::Resolved) |
                (ResolutionState::Resolved, ResolutionState::Stable) =>
                    ResolutionState::Stable,
                
                (ResolutionState::Resolved, ResolutionState::Resolved) =>
                    ResolutionState::Resolved,
            }
        };
    }


    pub fn debug_label(&self) -> &'static str
    {
        match self
        {
            ResolutionState::Resolved => "resolved",
            ResolutionState::Stable => "stable",
            ResolutionState::Unresolved => "unresolved",
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

        if resolution_state.is_stable_or_resolved()
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

    if resolution_state.is_stable_or_resolved()
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

                if is_last_iteration {
                    res::finalize_res(
                        report,
                        ast_res,
                        defs,
                        &ctx)?;
                }
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

                if is_last_iteration {
                    align::finalize_align(
                        report,
                        ast_align,
                        defs,
                        &ctx)?;
                }
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

                if is_last_iteration {
                    addr::finalize_addr(
                        report,
                        ast_addr,
                        defs,
                        &ctx)?;
                }
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

    if opts.debug_iterations
    {
        println!(
            "iteration result: {}\n",
            resolution_state.debug_label());
    }

    Ok(resolution_state)
}


pub fn handle_value_resolution(
    opts: &asm::AssemblyOptions,
    report: &mut diagn::Report,
    span: diagn::Span,
    can_guess: bool,
    is_guess: bool,
    is_stable: bool,
    is_resolved: &mut bool,
    suppress_diagn: bool,
    debug_element_type: &str,
    user_element_type: &str,
    element_name: Option<&str>,
    value: &expr::Value)
    -> Result<asm::ResolutionState, ()>
{
    if opts.debug_iterations
    {
        println!(" {}{:>5} `{}` = {}",
            if !is_guess && opts.optimize_statically_known { "🟢" }
                else if is_stable { "🔵" }
                else { "  " },
            debug_element_type,
            if let Some(name) = element_name { name.to_string() } else { format!("{:?}", span) },
            value);
    }

    if !is_guess
    {
        *is_resolved = true;
        return Ok(asm::ResolutionState::Resolved);
    }

    if is_stable
    {
        return Ok(asm::ResolutionState::Stable);
    }
    
    if !can_guess && !suppress_diagn {
        report.error_span(
            format!("{} did not converge", user_element_type),
            span);
    }

    Ok(asm::ResolutionState::Unresolved)
}