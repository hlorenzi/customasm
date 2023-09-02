#subruledef reg
{
	x{val: u4} => val`4
}

#ruledef test
{
	test {imm: u8} => imm
	testasm {R1: reg}, {R2: reg} => asm{test ({R1} @ {R2})}
}

testasm x0, x1 ; error: failed / note:_:9: within / note:_:9: test (x0 @ x1) / error:_:9: failed / note:_:8: within / error:_:9: unknown symbol `x0`