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

    generate_tests_from_folder(&mut f, &std::env::current_dir().unwrap().join("tests"));
}


fn generate_tests_from_folder(f: &mut dyn std::io::Write, folder: &std::path::Path)
{
    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());

        if path.is_file()
        {
            let contents = std::fs::read_to_string(&path).unwrap();
            //println!("{:?}: {:?}", &path, &contents);
            generate_tests_from_file(f, &path, &contents);
        }
        else
        {
            generate_tests_from_folder(f, &path);
        }
    }
}


fn generate_tests_from_file(f: &mut dyn std::io::Write, filepath: &std::path::PathBuf, contents: &str)
{
    let filename = filepath.file_stem().unwrap().to_string_lossy();
    let mut auto_subfile_index = 1;

    for line in contents.lines()
    {
        if line.starts_with("; :::")
        {
            let mut name = format!("{}", &line.get(5..).unwrap().trim());
            if name == "include"
            {
                continue;
            }

            if name.len() == 0
            {
                name = format!("{}", auto_subfile_index);
                auto_subfile_index += 1;
            }

            write!(f,
                "#[test]
                fn {}_{}()
                {{
                    test_subfile({:?}, {:?});
                }}",
                filename, name,
                filepath, name).unwrap();
        }
    }
}