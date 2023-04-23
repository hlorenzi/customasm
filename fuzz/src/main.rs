#[macro_use]
extern crate afl;
use customasm::*;


fn main()
{
    fuzz!(|data: &[u8]|
    {
        let mut report = diagn::Report::new();

        let virtual_filename = "fuzz";
        let mut fileserver = util::FileServerMock::new();
        fileserver.add(virtual_filename, data);

        let opts = asm::AssemblyOptions::new();
        
        let _assembly = asm::assemble(
            &mut report,
            &opts,
            &mut fileserver,
            &[virtual_filename]);
    });
}
