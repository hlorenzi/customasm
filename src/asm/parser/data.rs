use crate::*;


pub fn parse_directive_data(
    state: &mut asm::parser::State,
    elem_size: Option<usize>,
    tk_hash: &syntax::Token)
    -> Result<(), ()>
{
    let mut is_first = true;
    
    loop
    {
        let expr = expr::Expr::parse(&mut state.parser)?;
        let span = if is_first
            { expr.span().join(&tk_hash.span) }
        else
            { expr.span().clone() };
        
        let mut invocation = asm::Invocation
        {
            ctx: state.asm_state.get_ctx(&state),
            size_guess: 0,
            span,
            kind: asm::InvocationKind::Data(asm::DataInvocation
            {
                expr,
                elem_size,
            })
        };

        let resolved = state.asm_state.resolve_data_invocation(
            state.report.clone(),
            &invocation,
            state.fileserver,
            false);

        match elem_size
        {
            Some(elem_size) => invocation.size_guess = elem_size,
            None =>
            {
                invocation.size_guess = match resolved.map(|r| r.get_bigint())
                {
                    Ok(Some(bigint)) =>
                    {
                        match bigint.size
                        {
                            Some(size) => size,
                            None => 0,
                        }
                    }
                    _ => 0
                };
            }
        }

        let bankdata = state.asm_state.get_bankdata(state.asm_state.cur_bank);
        bankdata.check_writable(&state.asm_state, state.report.clone(), &invocation.span)?;
        
        let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
        bankdata.push_invocation(invocation);
        
        if state.parser.maybe_expect(syntax::TokenKind::Comma).is_none()
        {
            break;
        }

        if state.parser.next_is_linebreak()
        {
            break;
        }
            
        is_first = false;
    }

    Ok(())
}