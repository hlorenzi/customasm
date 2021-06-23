extern crate customasm;


fn main()
{
	split_tests();
	/*let args: Vec<String> = std::env::args().collect();
	
	let mut fileserver = customasm::util::FileServerReal::new();
	
	if let Err(()) = customasm::driver::drive(&args, &mut fileserver)
		{ std::process::exit(1); }*/
}




fn split_tests()
{
    let out_folder = std::env::current_dir().unwrap().join("tests2");
    split_tests_from_folder(&std::env::current_dir().unwrap().join("tests"), &out_folder);
}


fn split_tests_from_folder(folder: &std::path::Path, out_folder: &std::path::Path)
{
    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file()
        {
            let contents = std::fs::read_to_string(&path).unwrap();
            //println!("{:?}: {:?}", &path, &contents);
            split_tests_from_file(&path, &out_folder, &contents);
        }
        else
        {
            split_tests_from_folder(&path, &out_folder);
        }
    }
}


fn split_tests_from_file(filepath: &std::path::PathBuf, out_folder: &std::path::Path, contents: &str)
{
    let filename = filepath.file_stem().unwrap().to_string_lossy();
    let mut auto_subfile_index = 1;

    let lines = contents.lines().collect::<Vec<_>>();
    let mut i = 0;

	println!("{}.asm", &filename.as_ref());

	let mut cur_include_data = String::new();

    while i < lines.len()
    {
        if lines[i].starts_with("; :::")
        {
            let mut name = format!("{}", &lines[i].get(5..).unwrap().trim());

            if name.len() == 0
            {
                name = format!("{}.asm", auto_subfile_index);
                auto_subfile_index += 1;
            }

            let mut data = String::new();

			if name != "include"
			{
				data.push_str(&cur_include_data);
			}

            i += 1;
            while i < lines.len() && !lines[i].starts_with("; :::")
            {
                if !lines[i].starts_with("; ==")
                {
                    data.push_str(&lines[i].replace("error: :", "error:_:").replace("note: :", "note:_:"));
                    data.push_str("\n");
                }
                
                i += 1;
            }

			if name == "include"
			{
				cur_include_data = data;
				continue;
			}

            let mut out_path = out_folder.to_path_buf();
            out_path.push(&filename.as_ref());
            out_path.push(&name);

            println!("{}/{}.asm", &filename.as_ref(), name);

			std::fs::create_dir(out_path.parent().unwrap()).unwrap();
            std::fs::write(out_path, &data.trim()).unwrap();
        }
        else
        {
            i += 1;
        }
    }
}