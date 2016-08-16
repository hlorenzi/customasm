#![cfg(test)]


use util::expression::Expression;
use util::parser::Parser;
use util::tokenizer::tokenize;


fn pass(expr_str: &str, expected_result: i64)
{
	let tokens = tokenize("test", &expr_str.chars().collect::<Vec<char>>());
	let mut parser = Parser::new(&tokens);
	let expr = Expression::new_by_parsing(&mut parser).unwrap();
	let result = expr.resolve(&|_, _| panic!("invalid test")).unwrap().as_integer().unwrap();
	
	let result_i64 = result.to_i64().unwrap();
	
	if result_i64 != expected_result
	{
		println!("expected: {}", expected_result);
		println!("     got: {}", result_i64);
		panic!("expression failed but expected to pass");
	}
}


fn parse_fail(expr_str: &str, expected_error_substr: &str)
{
	let tokens = tokenize("test", &expr_str.chars().collect::<Vec<char>>());
	let mut parser = Parser::new(&tokens);
	match Expression::new_by_parsing(&mut parser)
	{
		Ok(_) => panic!("expression passed but error expected: {}", expr_str),
			
		Err(err) =>
			if !err.contains_str(expected_error_substr)
			{
				println!(" expected error msg: {}", expected_error_substr);
				println!("      got error msg: {}", err.get_msg());
				panic!("expression error mismatch");
			}
	}
}


#[test]
fn test_simple()
{
	pass("2 + 2", 4);
	pass("2 - 2", 0);
	pass("2 * 2", 4);
	pass("2 / 2", 1);
	
	pass("1 + 2 + 3", 6);
	pass("10 - 5 - 2", 3);
	pass("2 * 3 * 4", 24);
	pass("20 / 5 / 2", 2);
	
	pass(" 2 +  3  * 4 ", 14);
	pass(" 2 + (3  * 4)", 14);
	pass("(2 +  3) * 4 ", 20);
	
	pass("1 << 3", 8);
	pass("16 >> 3", 2);
	pass("1 << 2 + 1", 8);
	pass("16 >> 2 + 1", 2);
	pass("2 - 1 << 3", 8);
	pass("18 - 2 >> 3", 2);
	
	pass("0b0011 & 0b0101", 0b0001);
	pass("0b0011 | 0b0101", 0b0111);
	pass("0b0011 ^ 0b0101", 0b0110);
	
	pass("0b10110101[0:0]", 0b1);
	pass("0b10110101[1:0]", 0b01);
	pass("0b10110101[2:0]", 0b101);
	pass("0b10110101[3:0]", 0b0101);
	pass("0b10110101[4:0]", 0b10101);
	pass("0b10110101[5:0]", 0b110101);
	pass("0b10110101[6:0]", 0b0110101);
	pass("0b10110101[7:0]", 0b10110101);
	
	pass("0b10110101[1:1]", 0b0);
	pass("0b10110101[2:2]", 0b1);
	pass("0b10110101[3:3]", 0b0);
	pass("0b10110101[4:4]", 0b1);
	pass("0b10110101[5:5]", 0b1);
	pass("0b10110101[6:6]", 0b0);
	pass("0b10110101[7:7]", 0b1);
	
	pass("0b10110101[5:2]", 0b1101);
	
	parse_fail("8'0xfff", "not fit");
	parse_fail("64'0xff00ff00ff00ff00", "invalid");
	parse_fail("0x12[0:3]", "invalid slice");
	parse_fail("0x12[64:0]", "big slice");
	parse_fail("0x12[65:64]", "big slice");
}