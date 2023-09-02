#subruledef reg
{
	x{val: u4} => val`4
}

#ruledef test
{
	test {imm: u8} => imm
	testasm {R1: reg}, {R2: reg} => {
		temp = (R1 @ R2)
		asm{test {temp}}
	}
}

testasm x0, x1 ; = 0x01