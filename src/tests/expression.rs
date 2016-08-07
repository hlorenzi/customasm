#![cfg(test)]


use util::expression::Expression;
use util::parser::Parser;
use util::tokenizer::tokenize;


fn pass(expr_str: &str, expected_result: i64)
{
	let tokens = tokenize("test", &expr_str.chars().collect::<Vec<char>>());
	let mut parser = Parser::new("test", &tokens);
	let expr = Expression::new_by_parsing(&mut parser).unwrap();
	let result = expr.resolve(&|_, _| panic!("invalid test")).unwrap().as_i64().unwrap();
	
	if result != expected_result
	{
		panic!(format!(
			"\nexpression result mismatch: {}\n\n \
			expected: {}\n \
			.....got: {}\n",
			expr_str, expected_result, result));
	}
}


#[test]
fn test_expr()
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
	
	pass("0b0011 & 0b0101", 0b0001);
	pass("0b0011 | 0b0101", 0b0111);
	pass("0b0011 ^ 0b0101", 0b0110);
}