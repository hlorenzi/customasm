extern crate num;
extern crate getopts;


mod diagn;
mod syntax;
mod expr;
mod instrset;
mod asm;
mod util;
mod driver;


#[cfg(test)]
mod test;


pub use self::diagn::Report;
pub use self::instrset::read_instrset;
pub use self::asm::assemble;
pub use self::util::FileServer;
pub use self::util::FileServerMock;
pub use self::util::FileServerReal;
pub use self::driver::drive;