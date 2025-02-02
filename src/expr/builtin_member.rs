use crate::*;


pub fn resolve_builtin_member(
    query: &mut expr::EvalMemberQuery)
    -> Result<Option<expr::Value>, ()>
{
    match query.member_name
    {
        "size" =>
        {
            let (_bigint, size) = query.value.expect_sized_integerlike(
                query.report,
                query.span)?;
            
            Ok(Some(expr::Value::make_integer(size)))
        }

        _ => Ok(None)
    }
}