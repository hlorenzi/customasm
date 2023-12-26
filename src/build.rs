use vergen::{generate_cargo_keys, ConstantsFlags};

fn main()
{
    let mut flags = ConstantsFlags::empty();
    flags.toggle(ConstantsFlags::REBUILD_ON_HEAD_CHANGE);
    flags.toggle(ConstantsFlags::SEMVER_LIGHTWEIGHT);
    flags.toggle(ConstantsFlags::COMMIT_DATE);
    flags.toggle(ConstantsFlags::TARGET_TRIPLE);

    generate_cargo_keys(flags).expect("Unable to generate the cargo keys!");

    generate_std();
    generate_tests();
}

fn generate_std()
{
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("std_files.rs");
    let mut f = std::fs::File::create(&destination).unwrap();

    use std::io::Write;
    writeln!(f, "pub static STD_FILES: &[(&str, &str)] = &[").unwrap();

    generate_std_from_folder(
        &mut f,
        &std::env::current_dir().unwrap().join("std"),
        &"<std>/",
    );

    writeln!(f, "];").unwrap();
}

fn generate_std_from_folder(
    f: &mut dyn std::io::Write,
    folder: &std::path::Path,
    relative_path: &str,
)
{
    println!("cargo:rerun-if-changed={}", folder.to_string_lossy());

    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();

        let mut inner_relative_path = relative_path.to_string();
        inner_relative_path.push_str(&filename);

        if path.is_file()
        {
            println!("cargo:rerun-if-changed={}", path.to_string_lossy());

            let line = format!(
                "\t(\"{}\", include_str!(\"{}\")),",
                inner_relative_path,
                path.to_string_lossy().replace('\\', "/")
            );

            writeln!(f, "{}", line).unwrap();
        }
        else
        {
            inner_relative_path.push_str("/");

            generate_std_from_folder(f, &path, &inner_relative_path);
        }
    }
}

fn generate_tests()
{
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("test.rs");
    let mut f = std::fs::File::create(&destination).unwrap();

    generate_tests_from_folder(&mut f, &std::env::current_dir().unwrap().join("tests"), "");
}

fn generate_tests_from_folder(f: &mut dyn std::io::Write, folder: &std::path::Path, test_name: &str)
{
    println!("cargo:rerun-if-changed={}", folder.to_string_lossy());

    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();

        let file_stem = path.file_stem().unwrap().to_string_lossy();

        let mut new_test_name = test_name.to_string();
        new_test_name.push_str(&file_stem);

        if path.is_file()
        {
            println!("cargo:rerun-if-changed={}", path.to_string_lossy());

            let extension = path.extension().map_or("", |e| e.to_str().unwrap());

            if extension != "asm"
            {
                continue;
            }

            writeln!(f, "#[test]").unwrap();
            writeln!(f, "fn {}()", new_test_name.replace(".", "_")).unwrap();
            writeln!(f, "{{").unwrap();
            writeln!(f, "\ttest_file({:?});", path).unwrap();
            writeln!(f, "}}").unwrap();
            writeln!(f, "").unwrap();
        }
        else
        {
            new_test_name.push_str("_");

            generate_tests_from_folder(f, &path, &new_test_name);
        }
    }
}
