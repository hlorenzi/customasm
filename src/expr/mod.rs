mod expression;
pub use self::expression::{BinaryOp, Expr, ExprString, UnaryOp, Value};

mod parser;
pub use parser::{parse, parse_optional};

mod inspect;
pub use inspect::{
    StaticallyKnownFunctionQuery, StaticallyKnownLocal, StaticallyKnownProvider,
    StaticallyKnownVariableQuery,
};

mod eval;
pub use self::eval::{
    dummy_eval_asm, dummy_eval_fn, dummy_eval_query, dummy_eval_var, EvalAsmBlockQuery,
    EvalContext, EvalFunctionQuery, EvalProvider, EvalQuery, EvalVariableQuery,
};

mod builtin_fn;
pub use self::builtin_fn::{
    eval_builtin_fn, get_static_size_builtin_fn, get_statically_known_value_builtin_fn,
    resolve_builtin_fn,
};

pub const PARSE_RECURSION_DEPTH_MAX: usize = 50;
pub const EVAL_RECURSION_DEPTH_MAX: usize = 25;
