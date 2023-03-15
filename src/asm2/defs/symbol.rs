use crate::*;


#[derive(Debug)]
pub struct Symbol
{
    pub item_ref: util::ItemRef<Self>,
    pub value: Option<expr::Value>,
}