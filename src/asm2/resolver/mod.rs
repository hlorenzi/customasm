use crate::*;


mod constant;
pub use constant::{
    resolve_constants,
    resolve_constants_once,
    resolve_constant,
};


pub fn eval(
    report: &mut diagn::Report,
    decls: &asm2::ItemDecls,
    defs: &asm2::ItemDefs,
    symbol_ctx: &util::SymbolContext,
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

    expr.eval2(report, &mut provider)
}