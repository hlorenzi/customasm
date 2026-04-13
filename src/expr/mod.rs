mod expression;
pub use self::expression::{
    Expr,
    ExprStructMemberInit,
    Value,
    ValueMetadata,
    ValueStruct,
    ValueStructMember,
    UnaryOp,
    BinaryOp,
};

mod parser;
pub use parser::{
    parse,
    parse_optional,
    parse_optional_decimal_usize_greedy,
};

mod eval;
pub use self::eval::{
    EvalContext,
    EvalProvider,
    EvalQuery,
    EvalCtxLabelQuery,
    EvalVariableQuery,
    EvalMemberQuery,
    EvalFunctionQuery,
    EvalAsmBlockQuery,
    ASM_SUBSTITUTION_VARIABLE,
    dummy_eval_query,
    dummy_eval_ctxlabel,
    dummy_eval_var,
    dummy_eval_member,
    dummy_eval_fn,
    dummy_eval_asm,
};

mod inspect;
pub use self::inspect::{
    StaticallyKnownProvider,
    StaticallyKnownLocal,
};

mod builtin_member;
pub use self::builtin_member::resolve_builtin_member;

mod builtin_fn;
pub use self::builtin_fn::{
    ExprBuiltinFn,
    resolve_builtin_fn,
    get_builtin_fn_eval,
    get_builtin_fn_size_guess,
};


pub const PARSE_RECURSION_DEPTH_MAX: usize = 25;
pub const EVAL_RECURSION_DEPTH_MAX: usize = 25;