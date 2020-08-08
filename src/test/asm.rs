use crate::*;
use std;


#[test]
fn test_files()
{
    test_folder(&std::env::current_dir().unwrap().join("tests"));
}


fn test_folder(folder: &std::path::Path)
{
    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file()
        {
            let contents = std::fs::read_to_string(&path).unwrap();
            //println!("{:?}: {:?}", &path, &contents);
            test_file(&path.as_path().file_name().unwrap().to_string_lossy(), contents).unwrap();
        }
        else
        {
            test_folder(&path);
        }
    }
}


enum TestExpectation
{
    Pass
    {
        output: util::BitVec,
    },
    Error
    {
        line: usize,
        excerpt: String,
    }
}


fn test_file<T: Into<String>>(filename: &str, contents: T) -> Result<(), ()>
{
    let mut fileserver = util::FileServerMock::new();
    
    let contents = contents.into();
    let mut cur_subfile_name = None;
    let mut cur_subfile_contents = None;
    let mut cur_subfile_expectation = None;
    let mut auto_subfile_index = 0;

    let mut root_subfiles = Vec::new();
    let mut expectations = std::collections::HashMap::<String, TestExpectation>::new();

    let mut line_num = 0;

    for line in contents.lines()
    {
        if line.starts_with("---")
        {
            if cur_subfile_name.is_some()
            {
                //println!("-> {}\n\n{}\n\n", cur_subfile_name.as_ref().unwrap(), cur_subfile_contents.as_ref().unwrap());
                fileserver.add(cur_subfile_name.clone().unwrap(), cur_subfile_contents.unwrap());

                if let Some(expectation) = cur_subfile_expectation
                {
                    expectations.insert(cur_subfile_name.unwrap(), expectation);
                }
            }

            let mut name = line.get(3..).unwrap().trim().to_string();
            if name.len() == 0
            {
                name = format!("test_{}", auto_subfile_index);
                auto_subfile_index += 1;
                root_subfiles.push(name.clone());
            }
            //println!("found subfile: {}", &name);

            cur_subfile_name = Some(name);
            cur_subfile_contents = Some(String::new());
            cur_subfile_expectation = None;
            line_num = 0;
        }
        else
        {
            cur_subfile_contents.as_mut().unwrap().push_str(&line);
            cur_subfile_contents.as_mut().unwrap().push_str("\n");

            if let Some(value_index) = line.find("; = ")
            {
                let value_str = line.get((value_index + 4)..).unwrap().trim();
                let value = syntax::excerpt_as_bigint(None, value_str, &diagn::Span::new_dummy()).unwrap();
                
                if cur_subfile_expectation.is_none()
                {
                    cur_subfile_expectation = Some(TestExpectation::Pass
                    {
                        output: util::BitVec::new()
                    });
                }

                if let TestExpectation::Pass{ output: bitvec } = cur_subfile_expectation.as_mut().unwrap()
                {
                    let index = bitvec.len();
                    bitvec.write_bigint(index, value);
                }
            }
            else if let Some(value_index) = line.find("; error: ")
            {
                let excerpt = line.get((value_index + 9)..).unwrap().trim().to_string();
    
                cur_subfile_expectation = Some(TestExpectation::Error
                {
                    line: line_num,
                    excerpt,
                });
            }

            line_num += 1;
        }
    }

    if cur_subfile_name.is_some()
    {
        fileserver.add(cur_subfile_name.clone().unwrap(), cur_subfile_contents.unwrap());

        if let Some(expectation) = cur_subfile_expectation
        {
            expectations.insert(cur_subfile_name.unwrap(), expectation);
        }
    }

    for root_subfile in &root_subfiles
    {    
        let report = diagn::RcReport::new();
        let mut state = asm::State::new();

        if fileserver.exists("include")
        {
            let _ = state.process_file(report.clone(), &mut fileserver, "include");
        }

        let _ = state.process_file(report.clone(), &mut fileserver, root_subfile);
        let output = state.resolve_bank(report.clone(), &state.banks[0]);
        
        let mut msgs = Vec::<u8>::new();
        report.print_all(&mut msgs, &fileserver);
        print!("{:}", String::from_utf8(msgs).unwrap());

        if report.has_errors()
        {
            if let TestExpectation::Error{ excerpt, line } = expectations.get(root_subfile).unwrap()
            {
                if !report.has_first_error_at(&fileserver, root_subfile, *line, excerpt)
                {
                    println!("\n\
                        [{}: {}]\n\
                        > test failed -- error mismatch\n\
                        > expected: `{}` at line {}\n",
                        filename, &root_subfile,
                        excerpt, line);
                        
                    panic!("test failed");
                }
            }
            else
            {
                println!("\n\
                    [{}: {}]\n\
                    > test failed -- expected to pass but errored\n",
                    filename, &root_subfile);
                    
                panic!("test failed");
            }
        }
        else
        {
            let output = output.unwrap();

            if let TestExpectation::Pass{ output: expected_output } = expectations.get(root_subfile).unwrap()
            {
                if format!("{:x}", output) != format!("{:x}", expected_output)
                {
                    println!("\n\
                        [{}: {}]\n\
                        > test failed -- output mismatch\n\
                        > got:      0x{:x}\n\
                        > expected: 0x{:x}\n",
                        filename, &root_subfile,
                        &output, &expected_output);
                        
                    panic!("test failed");
                }
            }
            else
            {
                println!("\n\
                    [{}: {}]\n\
                    > test failed -- expected to error but passed\n",
                    filename, &root_subfile);
                    
                panic!("test failed");
            }
        }
    }

	Ok(())
}