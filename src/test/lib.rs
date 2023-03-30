use crate::*;
use sha2::*;

fn load_file_and_assemble_str_to_binary(filename: &str, hash: &[u8]) {
    let src = std::fs::read_to_string(filename).unwrap();
    let output = assemble_str_to_binary(&src);

    if let Some(binary) = output.0 {
        let mut output_hasher = sha2::Sha256::new();
        output_hasher.update(binary);
        let output_hash = output_hasher.finalize();

        assert_eq!(output_hash[..], *hash);
    } else {
        panic!("failed to assemble");
    }
}

#[test]
fn test_assemble_str_to_binary() {
    load_file_and_assemble_str_to_binary(
        "examples/basic.asm",
        &[
            70, 139, 45, 46, 111, 126, 164, 124, 241, 45, 193, 32, 116, 119, 229, 149, 159, 100,
            110, 138, 69, 217, 176, 220, 115, 186, 132, 102, 96, 201, 46, 16,
        ],
    );
}
