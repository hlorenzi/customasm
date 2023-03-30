use crate::*;

#[derive(Debug)]
pub struct Function {
    pub decl_span: diagn::Span,
    pub name: String,
    pub params: Vec<String>,
    pub body: expr::Expr,
}
