use crate::asm;
use crate::diagn;
use crate::util;
use std;


fn test_file<T: Into<String>>(str: T) -> Result<(), ()>
{
	let mut fileserver = util::FileServerMock::new();
	fileserver.add("str", str.into());
	
	let report = diagn::RcReport::new();
	let mut state = asm::State::new();
    let _ = state.process_file(report.clone(), &mut fileserver, "str");
    
	let mut msgs = Vec::<u8>::new();
	report.print_all(&mut msgs, &fileserver);
	print!("{}", String::from_utf8(msgs).unwrap());
	Ok(())
}


#[test]
fn test_files()
{
    let folder = std::env::current_dir().unwrap().join("tests");

    for entry in std::fs::read_dir(folder).unwrap()
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file()
        {
            let contents = std::fs::read_to_string(&path).unwrap();
            println!("{:?}: {:?}", &path, &contents);
            test_file(contents).unwrap();
        }
    }
}