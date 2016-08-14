#![cfg(test)]


use definition;


fn pass(def_str: &str)
{
	definition::parse("test", &def_str.chars().collect::<Vec<char>>()).unwrap();
}


fn fail(def_str: &str, expected_error_line: usize, expected_error_substr: &str)
{
	match definition::parse("test", &def_str.chars().collect::<Vec<char>>())
	{
		Ok(_) => panic!("test passed but error expected"),
			
		Err(err) =>
			if !err.line_is(expected_error_line) || !err.contains_str(expected_error_substr)
			{
				println!(" expected error msg: {}", expected_error_substr);
				println!("      got error msg: {}", err.get_msg());
				println!("expected error line: {}", expected_error_line);
				println!("     got error line: {}", err.get_line());
				panic!("test error mismatch");
			}
	}
}


#[test]
fn test_simple()
{	
	pass("");
	pass(".align 8");
	pass(".align 8 \n halt -> 8'0");
	pass(".align 8 \n halt -> 8'0x33");
	pass(".align 8 \n halt -> 4'0xd 4'0xa");
	
	pass(".align 8 \n halt         -> pc[7:0]");
	pass(".align 8 \n halt {a}     ->  a[7:0]");
	pass(".align 8 \n halt {a}     ->  a[15:0]");
	pass(".align 8 \n halt {a}     ->  a[3:0] a[3:0]");
	pass(".align 8 \n halt {a} {b} ->  a[7:0] b[7:0]");
	pass(".align 8 \n halt {a} {b} ->  a[3:0] b[3:0]");
	pass(".align 8 \n halt {a} {b} ->  a[7:0] a[7:0]");
	pass(".align 8 \n halt {a} {b} ->  b[7:0] a[7:0]");
	pass(".align 8 \n halt {a} {b} ->  b[7:0] b[7:0]");
	
	pass(".align 8 \n halt {a}     -> 8'0x45 a[7:0]");
	pass(".align 8 \n halt {a}     -> 4'0x7  a[7:0] 4'0x7");
	
	fail(".xyz 8", 1, "directive");
	fail(".align 8 \n -> 8'0", 2, "expected pattern");
	fail(".align 8 \n halt ->", 2, "expected expression");
	fail(".align 8 \n halt -> 0x12", 2, "explicit size");
	fail(".align 8 \n halt -> 8'0xfff", 2, "not fit");
	fail(".align 8 \n halt -> 64'0xff00ff00ff00ff00", 2, "invalid");
	fail(".align 8 \n halt -> 4'0xd", 2, "aligned");
	fail(".align 8 \n halt -> xyz", 2, "unknown");
	fail(".align 8 \n halt -> xyz[7:0]", 2, "unknown");
	fail(".align 8 \n halt {a} -> a", 2, "explicit size");
	fail(".align 8 \n halt {a} -> a[3:0]", 2, "aligned");
	fail(".align 8 \n halt {a} -> a[0:3]", 2, "invalid slice");
	fail(".align 8 \n halt {a} -> a[64:3]", 2, "big slice");
	fail(".align 8 \n halt {a} -> a[65:64]", 2, "big slice");
}