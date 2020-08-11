use crate::*;

//mod excerpt;
//mod expr;
//mod cpudef;
//mod asm;


// generated by build script
include!(concat!(env!("OUT_DIR"), "/test.rs"));


pub struct TestFile
{
    name: String,
    contents: String,
    expectation: Option<TestExpectation>,
}


pub enum TestExpectation
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


pub fn parse_subfiles<T: Into<String>>(contents: T, up_to_subfile: &str) -> Result<Vec<TestFile>, ()>
{
    let mut files = Vec::new();
    
    let contents = contents.into();
    let mut cur_subfile_name: Option<String> = None;
    let mut cur_subfile_contents = None;
    let mut cur_subfile_expectation = None;
    let mut auto_subfile_index = 1;

    let mut line_num = 0;

    for line in contents.lines()
    {
        if line.starts_with("; :::")
        {
            if let Some(cur_name) = cur_subfile_name
            {
				files.retain(|f: &TestFile| f.name != cur_name);
                files.push(TestFile
                {
                    name: cur_name.clone(),
                    contents: cur_subfile_contents.clone().unwrap(),
                    expectation: cur_subfile_expectation,
				});

				cur_subfile_name = None;
				cur_subfile_expectation = None;
				
				if cur_name == up_to_subfile
				{
					break;
				}
			}
			
            let mut name = format!("{}", &line.get(5..).unwrap().trim());

            if name.len() == 0
            {
                name = format!("{}", auto_subfile_index);
                auto_subfile_index += 1;
			}
			
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
        files.push(TestFile
        {
            name: cur_subfile_name.clone().unwrap(),
            contents: cur_subfile_contents.unwrap(),
            expectation: cur_subfile_expectation,
        });
    }

    Ok(files)
}


pub fn test_subfile(filepath: &str, subfilename: &str)
{
	let contents = std::fs::read_to_string(&filepath).unwrap();
	
	let subfiles = parse_subfiles(contents, subfilename).unwrap();
	let mut fileserver = util::FileServerMock::new();

	for file in &subfiles
	{
		fileserver.add(file.name.clone(), file.contents.clone());
	}

	let subfile = subfiles.iter().find(|f| f.name == subfilename).unwrap();

	let report = diagn::RcReport::new();
	let mut state = asm::State::new();

	if fileserver.exists("include")
	{
		let _ = state.process_file(report.clone(), &mut fileserver, "include");
	}

	let _ = state.process_file(report.clone(), &mut fileserver, subfilename);
	let output = state.resolve_bank(report.clone(), &state.banks[0]);
	
	let mut msgs = Vec::<u8>::new();
	report.print_all(&mut msgs, &fileserver);
	print!("{:}", String::from_utf8(msgs).unwrap());

	if report.has_errors()
	{
		if let TestExpectation::Error{ excerpt, line } = subfile.expectation.as_ref().unwrap()
		{
			if !report.has_first_error_at(&fileserver, &subfile.name, *line, &excerpt)
			{
				println!("\n\
					> test failed -- error mismatch\n\
					> expected: `{}` at line {}\n",
					excerpt, line);
					
				panic!("test failed");
			}
		}
		else
		{
			println!("\n\
				> test failed -- expected to pass but errored\n");
				
			panic!("test failed");
		}
	}
	else
	{
		let output = output.unwrap();

		if let TestExpectation::Pass{ output: expected_output } = subfile.expectation.as_ref().unwrap()
		{
			if format!("{:x}", output) != format!("{:x}", expected_output)
			{
				println!("\n\
					> test failed -- output mismatch\n\
					> got:      0x{:x}\n\
					> expected: 0x{:x}\n",
					&output, &expected_output);
					
				panic!("test failed");
			}
		}
		else
		{
			println!("\n\
				> test failed -- expected to error but passed\n");
				
			panic!("test failed");
		}
	}
}