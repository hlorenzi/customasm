mod token;
pub use self::token::{
    Token,
    TokenKind,
    decide_next_token,
    is_whitespace,
    is_dec_number_start,
    is_dec_number_mid,
};

mod walker;
pub use self::walker::Walker;

mod excerpt;
pub use self::excerpt::{
    unescape_string,
    excerpt_as_string_contents,
    excerpt_as_usize,
    excerpt_as_bigint,
};