use crate::*;


pub fn resolve_builtin_member(
    query: &mut expr::EvalMemberQuery)
    -> Result<Option<expr::Value>, ()>
{
    if let expr::Value::Struct(_, data) = &query.value
    {
        if let Some(member) = data.members.iter().find(|m| m.name == query.member_name)
        {
            return Ok(Some(member.value.clone()));
        }
        else
        {
            return Ok(None);
        }
    }

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