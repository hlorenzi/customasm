use crate::*;


mod iter;
pub use iter::{
    AstIteratorWithContext,
    AstIteratorWithContextItem,
    iter_with_context,
};

mod constant;
pub use constant::{
    resolve_constants,
    resolve_constants_once,
    resolve_constant,
};

mod instruction;
pub use instruction::{
    resolve_instruction,
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


pub fn resolve_once(
    report: &mut diagn::Report,
    ast: &asm2::AstTopLevel,
    decls: &asm2::ItemDecls,
    defs: &mut asm2::ItemDefs)
    -> Result<asm2::ResolutionState, ()>
{
    println!("== resolve_once ==");
    let mut resolution_state = asm2::ResolutionState::Resolved;

    for item in asm2::resolver::iter_with_context(ast, decls)
    {
        if let asm2::AstAny::Symbol(ast_symbol) = item.node
        {
            resolution_state.merge(
                resolve_constant(
                    report,
                    ast_symbol,
                    decls,
                    defs,
                    &item.get_symbol_ctx())?);
        }
        
        else if let asm2::AstAny::Instruction(ast_instr) = item.node
        {
            resolution_state.merge(
                resolve_instruction(
                    report,
                    ast_instr,
                    decls,
                    defs,
                    &item.get_symbol_ctx())?);
        }
    }

    Ok(resolution_state)
}


pub fn eval(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    symbol_ctx: &util::SymbolContext,
    eval_ctx: &mut expr::EvalContext2,
    expr: &expr::Expr)
    -> Result<expr::Value, ()>
{
    let mut eval_var = |info: &mut expr::EvalVariableInfo2|
    {
        let symbol_ref = decls.symbols.get_by_name(
            info.report,
            info.span,
            symbol_ctx,
            info.hierarchy_level,
            info.hierarchy)?;

        let symbol = defs.symbols.get(symbol_ref);
        Ok(symbol.value.clone())
    };

    let mut provider = expr::EvalProvider {
        eval_var: &mut eval_var,
        eval_fn: &mut expr::dummy_eval_fn(),
        eval_asm: &mut expr::dummy_eval_asm(),
    };

    expr.eval2_with_ctx(
        report,
        eval_ctx,
        &mut provider)
}