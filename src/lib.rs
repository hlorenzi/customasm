mod parser;
mod parser_tests;

pub mod configuration;
pub mod translator;

pub use configuration::Configuration;
pub use translator::translate;