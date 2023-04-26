mod expression;
pub use self::expression::{
    Expr,
    Value,
    ExprString,
    UnaryOp,
    BinaryOp,
};

mod parser;
pub use parser::{
    parse,
    parse_optional,
};

mod inspect;
pub use inspect::{
    StaticallyKnownProvider,
    StaticallyKnownVariableQuery,
    StaticallyKnownLocal,
};

mod eval;
pub use self::eval::{
    EvalContext,
    EvalProvider,
    EvalQuery,
    EvalVariableQuery,
    EvalFunctionQuery,
    EvalAsmBlockQuery,
    dummy_eval_query,
    dummy_eval_var,
    dummy_eval_fn,
    dummy_eval_asm,
};

mod builtin_fn;
pub use self::builtin_fn::{
    resolve_builtin_fn,
    eval_builtin_fn,
    get_static_size_builtin_fn,
    get_statically_known_value_builtin_fn,
};