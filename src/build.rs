extern crate vergen;


use vergen::{ConstantsFlags, generate_cargo_keys};


fn main()
{
    let mut flags = ConstantsFlags::empty();
    flags.toggle(ConstantsFlags::REBUILD_ON_HEAD_CHANGE);
    flags.toggle(ConstantsFlags::SEMVER_LIGHTWEIGHT);
    flags.toggle(ConstantsFlags::COMMIT_DATE);
    flags.toggle(ConstantsFlags::TARGET_TRIPLE);
    
    generate_cargo_keys(flags).expect("Unable to generate the cargo keys!");

    generate_tests();
}


fn generate_tests()
{
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("test.rs");
    println!("{:?}", destination);
    let mut f = std::fs::File::create(&destination).unwrap();

    generate_tests_from_folder(
        &mut f,
        &std::env::current_dir().unwrap().join("tests"),
        &String::new());
}


fn generate_tests_from_folder(f: &mut dyn std::io::Write, folder: &std::path::Path, test_name: &String)
{
    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_stem = path.file_stem().unwrap().to_string_lossy();
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());

        if path.is_file()
        {
            let mut new_test_name = test_name.clone();
            new_test_name.push_str(&file_stem);
            
            write!(f,
                "#[test]
                fn {}()
                {{
                    test_file({:?});
                }}",
                new_test_name.replace(".", "_"),
                path).unwrap();
        }
        else
        {
            let mut new_test_name = test_name.clone();
            new_test_name.push_str(&file_stem);
            new_test_name.push_str("_");

            generate_tests_from_folder(f, &path, &new_test_name);
        }
    }
}