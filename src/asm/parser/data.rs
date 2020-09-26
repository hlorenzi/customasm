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
            { expr.span() };
        
        let mut invokation = asm::Invokation
        {
            ctx: state.asm_state.get_ctx(),
            size_guess: 0,
            span,
            kind: asm::InvokationKind::Data(asm::DataInvokation
            {
                expr,
                elem_size,
            })
        };

        let resolved = state.asm_state.resolve_data_invokation(
            state.report.clone(),
            &invokation,
            false);

        match elem_size
        {
            Some(elem_size) => invokation.size_guess = elem_size,
            None =>
            {
                invokation.size_guess = match resolved
                {
                    Ok(expr::Value::Integer(bigint)) =>
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
        bankdata.check_writable(&state.asm_state, state.report.clone(), &invokation.span)?;
        
        let bankdata = state.asm_state.get_bankdata_mut(state.asm_state.cur_bank);
        bankdata.push_invokation(invokation);
        
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