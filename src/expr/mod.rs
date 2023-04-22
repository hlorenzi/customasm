mod expression;
mod parser;
mod parser2;
mod eval;

mod inspect;
pub use inspect::{
    StaticSizeInfo,
};

mod eval2;
pub use self::eval2::{
    EvalContext2,
    EvalProvider,
    EvalVariableInfo2,
    EvalFunctionInfo2,
    EvalAsmInfo2,
    dummy_eval_var,
    dummy_eval_fn,
    dummy_eval_asm,
};

mod builtin_fn;
pub use self::builtin_fn::{
    resolve_builtin_fn,
    eval_builtin_fn,
    get_static_size_builtin_fn,
};


pub use self::expression::Expr;
pub use self::expression::Value;
pub use self::expression::ValueString;
pub use self::expression::UnaryOp;
pub use self::expression::BinaryOp;
pub use self::eval::EvalContext;
pub use self::eval::EvalVariableInfo;
pub use self::eval::EvalFunctionInfo;
pub use self::eval::EvalAsmInfo;
pub use self::parser2::parse;
pub use self::parser2::parse_optional;