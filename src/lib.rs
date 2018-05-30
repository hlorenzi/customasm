extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;
extern crate getopts;


mod diagn;
mod syntax;
mod expr;
mod asm;
mod util;
mod driver;


#[cfg(test)]
mod test;


pub use self::diagn::Report;
pub use self::asm::AssemblerState;
pub use self::util::FileServer;
pub use self::util::FileServerMock;
pub use self::util::FileServerReal;
pub use self::driver::drive;


#[no_mangle]
pub extern fn add(x: u32, y: u32) -> u32
{
    x + y
}