use crate::*;


pub fn parse_symbol(
    state: &mut asm::parser::State)
    -> Result<(), ()>
{
    let mut span = diagn::Span::new_dummy();
    let mut hierarchy_level = 0;
    
    while let Some(tk_dot) = state.parser.maybe_expect(syntax::TokenKind::Dot)
    {
        hierarchy_level += 1;
        span = span.join(&tk_dot.span);
    }

    let tk_name = state.parser.expect(syntax::TokenKind::Identifier)?;
    let name = tk_name.excerpt.clone().unwrap();
    span = span.join(&tk_name.span);

    let ctx;
    let kind;
    
    let value = if state.parser.maybe_expect(syntax::TokenKind::Equal).is_some()
    {
        kind = asm::SymbolKind::Constant;
        ctx = state.asm_state.get_ctx(state);
        let expr = expr::Expr::parse(&mut state.parser)?;
        let value = state.asm_state.eval_expr(
            state.report.clone(),
            &expr,
            &ctx,
            &mut expr::EvalContext::new(),
            state.fileserver,
            true)?;

        state.parser.expect_linebreak()?;
        value
    }
    else
    {
        kind = asm::SymbolKind::Label;

        if hierarchy_level == 0 && state.asm_state.cur_labelalign != 0
        {
            let bankdata = state.asm_state.get_bankdata(state.asm_state.cur_bank);
            let skip_bits = bankdata.bits_until_aligned(
                state.asm_state,
                state.asm_state.cur_labelalign);
        
            let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
            bankdata.reserve(skip_bits);
        }

        let tk_colon = state.parser.expect(syntax::TokenKind::Colon)?;
        
        span = span.join(&tk_colon.span);
        
        ctx = state.asm_state.get_ctx(state);
        let addr = state.asm_state.get_addr(
            state.report.clone(),
            &ctx,
            &span)?;
        
        let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
        bankdata.push_invocation(asm::Invocation
        {
            ctx: ctx.clone(),
            size_guess: 0,
            span: span.clone(),
            kind: asm::InvocationKind::Label(asm::LabelInvocation)
        });
        
        expr::Value::make_integer(addr)
    };

    state.asm_state.symbols.create(
        &ctx.symbol_ctx, 
        name, 
        hierarchy_level,
        kind,
        value,
        state.asm_state.cur_bank,
        state.report.clone(), 
        &span)?;

    Ok(())
}