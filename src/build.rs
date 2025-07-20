use std::process::Command;


fn main()
{
    println!(
        "cargo:rustc-env=CUSTOMASM_VERSION=v{}",
        env!("CARGO_PKG_VERSION"));
        
    println!(
        "cargo:rustc-env=CUSTOMASM_TARGET={}",
        std::env::var("TARGET").unwrap());

    generate_git_info();
    generate_std();
    generate_tests();
}


pub fn generate_git_info()
{
    let mut command_hash = Command::new("git");
    command_hash
        .arg("show")
        .arg("--pretty=format:%h")
        .arg("--no-patch");

    let hash = run_command_and_get_output(&mut command_hash)
        .unwrap_or("??????".to_string());

    println!("cargo:rustc-env=CUSTOMASM_COMMIT_HASH={}", hash);

    let mut command_date = Command::new("git");
    command_date
        .arg("show")
        .arg("--pretty=format:%cs")
        .arg("--no-patch");

    let date = run_command_and_get_output(&mut command_date)
        .unwrap_or("????-??-??".to_string());
    
    println!("cargo:rustc-env=CUSTOMASM_COMMIT_DATE={}", date);
}


fn run_command_and_get_output(
    command: &mut std::process::Command)
    -> Result<String, ()>
{
	let output = command
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()
		.map_err(|_| ())?
		.wait_with_output()
		.map_err(|_| ())?;

    if output.status.success()
    {
        return Ok(String::from_utf8(output.stdout).map_err(|_| ())?);
    }

    Err(())
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
        &"<std>/");
        
    writeln!(f, "];").unwrap();
}


fn generate_std_from_folder(
    f: &mut dyn std::io::Write,
    folder: &std::path::Path,
    relative_path: &str)
{
    println!("cargo:rerun-if-changed={}", folder.to_string_lossy());

    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();
        let filename = path
            .file_name()
            .unwrap()
            .to_string_lossy();

        let mut inner_relative_path = relative_path.to_string();
        inner_relative_path.push_str(&filename);

        if path.is_file()
        {
            println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    
            let line = format!("\t(\"{}\", include_str!(\"{}\")),",
                inner_relative_path,
                path.to_string_lossy().replace('\\', "/"));
            
            writeln!(f, "{}", line).unwrap();
        }
        else
        {
            inner_relative_path.push_str("/");

            generate_std_from_folder(
                f,
                &path,
                &inner_relative_path);
        }
    }
}


fn generate_tests()
{
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("test.rs");
    let mut f = std::fs::File::create(&destination).unwrap();

    generate_tests_from_folder(
        &mut f,
        &std::env::current_dir().unwrap().join("tests"),
        "");
}


fn generate_tests_from_folder(
    f: &mut dyn std::io::Write,
    folder: &std::path::Path,
    test_name: &str)
{
    println!("cargo:rerun-if-changed={}", folder.to_string_lossy());

    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();

        let file_stem = path
            .file_stem()
            .unwrap()
            .to_string_lossy();

        let mut new_test_name = test_name.to_string();
        new_test_name.push_str(&file_stem);

        if path.is_file()
        {
            println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    
            let extension = path
                .extension()
                .map_or("", |e| e.to_str().unwrap());

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

            generate_tests_from_folder(
                f,
                &path,
                &new_test_name);
        }
    }
}