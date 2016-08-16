#![cfg(test)]


use definition::Definition;


fn pass(def_str: &str)
{
	match Definition::from_str(def_str)
	{
		Ok(_) => { }
		Err(_) => panic!("definition failed but expected to pass")
	}
}


fn fail(def_str: &str, expected_error_line: usize, expected_error_substr: &str)
{
	match Definition::from_str(def_str)
	{
		Ok(_) => panic!("definition passed but error expected"),
			
		Err(err) =>
			if !err.line_is(expected_error_line) || !err.contains_str(expected_error_substr)
			{
				println!(" expected error msg: {}", expected_error_substr);
				println!("      got error msg: {}", err.get_msg());
				println!("expected error line: {}", expected_error_line);
				println!("     got error line: {}", err.get_line());
				panic!("definition error mismatch");
			}
	}
}


#[test]
fn test_simple()
{	
	pass("");
	pass(".align 8");
	pass(".align 8 \n halt -> 8'0");
	pass(".align 8 \n halt -> 8'0x3");
	pass(".align 8 \n halt -> 8'0x33");
	pass(".align 8 \n halt -> 4'0xd 4'0xa");
	
	pass("\n .align 8 \n    halt -> 8'0x3 \n    nop -> 8'0x6");
	pass("   .align 8 \n \n halt -> 8'0x3 \n    nop -> 8'0x6");
	pass("   .align 8 \n    halt -> 8'0x3 \n \n nop -> 8'0x6");
	pass("   .align 8 \n    halt -> 8'0x3 \n    nop -> 8'0x6 \n");
	pass("\n .align 8 \n \n halt -> 8'0x3 \n \n nop -> 8'0x6 \n");
	
	pass("; comment \n    .align 8           \n    halt -> 8'0x3           \n    nop -> 8'0x6");
	pass("                .align 8 ; comment \n    halt -> 8'0x3           \n    nop -> 8'0x6");
	pass("                .align 8           \n    halt -> 8'0x3 ; comment \n    nop -> 8'0x6");
	pass("                .align 8           \n    halt -> 8'0x3           \n    nop -> 8'0x6 ; comment");
	pass("; comment \n    .align 8 ; comment \n    halt -> 8'0x3 ; comment \n    nop -> 8'0x6 ; comment");
	
	pass("; comment \n \n .align 8           \n    halt -> 8'0x3           \n    nop -> 8'0x6");
	pass("                .align 8 ; comment \n \n halt -> 8'0x3           \n    nop -> 8'0x6");
	pass("                .align 8           \n    halt -> 8'0x3 ; comment \n \n nop -> 8'0x6");
	pass("                .align 8           \n    halt -> 8'0x3           \n    nop -> 8'0x6 ; comment \n");
	pass("                .align 8           \n    halt -> 8'0x3           \n    nop -> 8'0x6 ; comment \n");
	pass("; comment \n \n .align 8 ; comment \n \n halt -> 8'0x3 ; comment \n \n nop -> 8'0x6 ; comment \n");
	
	pass(".align 8 \n halt         -> pc[7:0]");
	pass(".align 8 \n halt {a}     ->  a[7:0]");
	pass(".align 8 \n halt {a}     ->  a[15:0]");
	pass(".align 8 \n halt {a}     ->  (a + 3)[7:0]");
	pass(".align 8 \n halt {a}     ->  a[3:0] a[3:0]");
	pass(".align 8 \n halt {a} {b} ->  a[7:0] b[7:0]");
	pass(".align 8 \n halt {a} {b} ->  a[3:0] b[3:0]");
	pass(".align 8 \n halt {a} {b} ->  a[7:0] a[7:0]");
	pass(".align 8 \n halt {a} {b} ->  b[7:0] a[7:0]");
	pass(".align 8 \n halt {a} {b} ->  b[7:0] b[7:0]");
	
	pass(".align 8 \n halt {a}     -> 8'0x45 a[7:0]");
	pass(".align 8 \n halt {a}     -> 4'0x7  a[7:0] 4'0x7");
	
	pass(".align 8 \n halt {a:  _      <= 0xff} -> 8'0x45 a[7:0]");
	pass(".align 8 \n halt {a!: _      <= 0xff} -> 8'0x45 a[7:0]");
	pass(".align 8 \n halt {a:  pc     <= 0xff} -> 8'0x45 a[7:0] pc[7:0]");
	pass(".align 8 \n halt {a:  pc + _ <= 0xff} -> 8'0x45 a[7:0] pc[7:0]");
	
	fail(".xyz 8", 1, "directive");
	fail(".align 8 .align 8", 1, "expected line break");
	fail(".align 8 \n -> 8'0", 2, "expected pattern");
	fail(".align 8 \n halt ->", 2, "expected expression");
	fail(".align 8 \n halt -> ; 8'0x3", 2, "expected expression");
	fail(".align 8 \n halt -> 64'0xff00ff00ff00ff00", 2, "invalid");
	fail(".align 8 \n halt -> 0x12", 2, "explicit size");
	fail(".align 8 \n halt -> 8'0xfff", 2, "not fit");
	fail(".align 8 \n halt -> 4'0xd", 2, "aligned");
	fail(".align 8 \n halt {a} -> a", 2, "explicit size");
	fail(".align 8 \n halt {a} -> a[3:0]", 2, "aligned");
	fail(".align 8 \n halt {a} -> a[0:3]", 2, "invalid slice");
	fail(".align 8 \n halt {a} -> a[64:3]", 2, "big slice");
	fail(".align 8 \n halt {a} -> a[65:64]", 2, "big slice");
	fail(".align 8 \n halt {a: a   <= 0xff} ->  a[7:0]", 2, "invalid variable");
	fail(".align 8 \n halt {a: xyz <= 0xff} ->  a[7:0]", 2, "invalid variable");
	fail(".align 8 \n halt {a: 'a  <= 0xff} ->  a[7:0]", 2, "invalid variable");
	fail(".align 8 \n halt {a: _   <= 0xff} -> 'a[7:0]", 2, "invalid variable");
	fail(".align 8 \n halt                  ->  xyz",    2, "unknown parameter");
	fail(".align 8 \n halt {a: _   <= 0xff} ->  _[7:0]", 2, "unknown parameter");
	fail(".align 8 \n halt {a: _   <= 0xff} ->  b[7:0]", 2, "unknown parameter");
}