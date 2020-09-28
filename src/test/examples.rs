use crate::*;
use sha2::*;


fn test_example(filename: &str, hash: &[u8])
{
    let mut fileserver = util::FileServerReal::new();
	let report = diagn::RcReport::new();
	let mut assembler = asm::Assembler::new();
    assembler.register_file(filename);
    
    let output = assembler.assemble(report.clone(), &mut fileserver, 2);
    report.print_all(&mut std::io::stdout(), &fileserver);

    let mut output_hasher = sha2::Sha256::new();
    output_hasher.update(output.as_ref().unwrap().format_binary());
    let output_hash = output_hasher.finalize();

    println!("{}", output.as_ref().unwrap().format_annotated_hex(&fileserver));
    
    assert_eq!(output_hash[..], *hash);
}


#[test]
fn test_nes_example()
{
    test_example(
        "examples/nes/main.asm",
        &[
            194, 190, 155, 144, 20, 50, 124, 170, 98, 249,
            99, 13, 159, 227, 169, 45, 203, 83, 54, 116, 56,
            113, 142, 114, 183, 67, 237, 97, 156, 21, 234, 191
        ]);
}


#[test]
fn test_basic_example()
{
    test_example(
        "examples/basic.asm",
        &[
            70, 139, 45, 46, 111, 126, 164, 124, 241, 45, 193,
            32, 116, 119, 229, 149, 159, 100, 110, 138, 69, 217,
            176, 220, 115, 186, 132, 102, 96, 201, 46, 16
        ]);
}