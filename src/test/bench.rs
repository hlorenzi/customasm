use crate::*;
extern crate test;


#[bench]
fn bench_nes_example(b: &mut test::Bencher)
{
    b.iter(|| {
        let mut report = diagn::Report::new();

        // TODO: Use a FileServerMock to avoid file IO
        let mut fileserver = util::FileServerReal::new();

        let opts = asm::AssemblyOptions::new();
        
        let assembly = asm::assemble(
            &mut report,
            &opts,
            &mut fileserver,
            &["examples/nes/main.asm"]);

        assembly
    });
}