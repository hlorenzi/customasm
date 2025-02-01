use crate::*;


#[test]
fn test_file_navigation()
{
    test("main.asm", "", Err(()));
    test("main.asm", ".", Err(()));
    test("main.asm", "sibling.asm", Ok("sibling.asm"));
    test("main.asm", "./sibling.asm", Ok("sibling.asm"));
    test("main.asm", "folder/inner.asm", Ok("folder/inner.asm"));
    test("main.asm", "./folder/inner.asm", Ok("folder/inner.asm"));
    test("main.asm", "..", Err(()));
    test("main.asm", "../outer.asm", Err(()));
    
    test("./main.asm", "", Err(()));
    test("./main.asm", ".", Err(()));
    test("./main.asm", "sibling.asm", Ok("./sibling.asm"));
    test("./main.asm", "./sibling.asm", Ok("./sibling.asm"));
    test("./main.asm", "folder/inner.asm", Ok("./folder/inner.asm"));
    test("./main.asm", "./folder/inner.asm", Ok("./folder/inner.asm"));
    test("./main.asm", "..", Err(()));
    test("./main.asm", "../outer.asm", Ok("outer.asm"));

    test("/main.asm", "", Err(()));
    test("/main.asm", ".", Err(()));
    test("/main.asm", "sibling.asm", Ok("/sibling.asm"));
    test("/main.asm", "./sibling.asm", Ok("/sibling.asm"));
    test("/main.asm", "folder/inner.asm", Ok("/folder/inner.asm"));
    test("/main.asm", "./folder/inner.asm", Ok("/folder/inner.asm"));
    test("/main.asm", "..", Err(()));
    test("/main.asm", "../outer.asm", Ok("outer.asm"));

    test("C:/main.asm", "", Err(()));
    test("C:/main.asm", ".", Err(()));
    test("C:/main.asm", "sibling.asm", Ok("C:/sibling.asm"));
    test("C:/main.asm", "./sibling.asm", Ok("C:/sibling.asm"));
    test("C:/main.asm", "folder/inner.asm", Ok("C:/folder/inner.asm"));
    test("C:/main.asm", "./folder/inner.asm", Ok("C:/folder/inner.asm"));
    test("C:/main.asm", "..", Err(()));
    test("C:/main.asm", "../outer.asm", Ok("outer.asm"));

    test("folder/inner.asm", "", Err(()));
    test("folder/inner.asm", ".", Err(()));
    test("folder/inner.asm", "sibling.asm", Ok("folder/sibling.asm"));
    test("folder/inner.asm", "./sibling.asm", Ok("folder/sibling.asm"));
    test("folder/inner.asm", "folder/inner.asm", Ok("folder/folder/inner.asm"));
    test("folder/inner.asm", "./folder/inner.asm", Ok("folder/folder/inner.asm"));
    test("folder/inner.asm", "..", Err(()));
    test("folder/inner.asm", "../outer.asm", Ok("outer.asm"));
    
    test("./folder/inner.asm", "", Err(()));
    test("./folder/inner.asm", ".", Err(()));
    test("./folder/inner.asm", "sibling.asm", Ok("./folder/sibling.asm"));
    test("./folder/inner.asm", "./sibling.asm", Ok("./folder/sibling.asm"));
    test("./folder/inner.asm", "folder/inner.asm", Ok("./folder/folder/inner.asm"));
    test("./folder/inner.asm", "./folder/inner.asm", Ok("./folder/folder/inner.asm"));
    test("./folder/inner.asm", "..", Err(()));
    test("./folder/inner.asm", "../outer.asm", Ok("./outer.asm"));
    test("./folder/inner.asm", "../../outer.asm", Ok("outer.asm"));
    
    test("/folder/inner.asm", "", Err(()));
    test("/folder/inner.asm", ".", Err(()));
    test("/folder/inner.asm", "sibling.asm", Ok("/folder/sibling.asm"));
    test("/folder/inner.asm", "./sibling.asm", Ok("/folder/sibling.asm"));
    test("/folder/inner.asm", "folder/inner.asm", Ok("/folder/folder/inner.asm"));
    test("/folder/inner.asm", "./folder/inner.asm", Ok("/folder/folder/inner.asm"));
    test("/folder/inner.asm", "..", Err(()));
    test("/folder/inner.asm", "../outer.asm", Ok("/outer.asm"));
    test("/folder/inner.asm", "../../outer.asm", Ok("outer.asm"));
    
    test("C:/folder/inner.asm", "", Err(()));
    test("C:/folder/inner.asm", ".", Err(()));
    test("C:/folder/inner.asm", "sibling.asm", Ok("C:/folder/sibling.asm"));
    test("C:/folder/inner.asm", "./sibling.asm", Ok("C:/folder/sibling.asm"));
    test("C:/folder/inner.asm", "folder/inner.asm", Ok("C:/folder/folder/inner.asm"));
    test("C:/folder/inner.asm", "./folder/inner.asm", Ok("C:/folder/folder/inner.asm"));
    test("C:/folder/inner.asm", "..", Ok("C:"));
    test("C:/folder/inner.asm", "../outer.asm", Ok("C:/outer.asm"));
    test("C:/folder/inner.asm", "../../outer.asm", Ok("outer.asm"));
    
    test("folder/subfolder/inner.asm", "", Err(()));
    test("folder/subfolder/inner.asm", ".", Err(()));
    test("folder/subfolder/inner.asm", "sibling.asm", Ok("folder/subfolder/sibling.asm"));
    test("folder/subfolder/inner.asm", "./sibling.asm", Ok("folder/subfolder/sibling.asm"));
    test("folder/subfolder/inner.asm", "folder/inner.asm", Ok("folder/subfolder/folder/inner.asm"));
    test("folder/subfolder/inner.asm", "./folder/inner.asm", Ok("folder/subfolder/folder/inner.asm"));
    test("folder/subfolder/inner.asm", "..", Ok("folder"));
    test("folder/subfolder/inner.asm", "../outer.asm", Ok("folder/outer.asm"));
    test("folder/subfolder/inner.asm", "../../outer.asm", Ok("outer.asm"));
    test("folder/subfolder/inner.asm", "../folder/outer.asm", Ok("folder/folder/outer.asm"));
}


fn test(base: &str, relative: &str, expected: Result<&str, ()>)
{
    let mut report = diagn::Report::new();
    let span = diagn::Span::new_dummy();

    let result = util::filename_navigate(
        &mut report, span, base, relative);

    assert_eq!(result.as_deref(), expected.as_deref());
}