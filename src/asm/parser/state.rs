use crate::*;


pub struct State<'a>
{
    pub report: diagn::RcReport,
    pub asm_state: &'a mut asm::State,
    pub fileserver: &'a dyn util::FileServer,
    pub filename: String,

    pub parser: syntax::Parser,
}