use crate::*;
use sha2::*;


fn test_example(filename: &str, hash: &[u8])
{
	let mut report = diagn::Report::new();
    let mut fileserver = util::FileServerReal::new();

    let opts = asm::AssemblyOptions::new();
    
	let assembly = asm::assemble(
        &mut report,
        &opts,
        &mut fileserver,
        &[filename]);

    report.print_all(
        &mut std::io::stdout(),
        &fileserver);

    let output = assembly.output.unwrap();

    let mut output_hasher = sha2::Sha256::new();
    output_hasher.update(output.format_binary());
    let output_hash = output_hasher.finalize();

    println!(
        "{}",
        output.format_annotated_hex(&fileserver));
    
    assert_eq!(output_hash[..], *hash);
}


#[test]
fn test_nes_example()
{
    test_example(
        "examples/nes/main.asm",
        &[
            226, 68, 213, 226, 71, 200, 16, 113, 21, 132,
            193, 34, 10, 134, 112, 238, 69, 165, 45, 199, 40,
            151, 195, 76, 157, 120, 172, 169, 37, 180, 123, 104
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