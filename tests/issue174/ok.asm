#ruledef
{
	test8_be  a, {val: i8}  => val
	test32_be a, {val: i32} => val
	test8_le  a, {val: i8}  => le(val)
	test32_le a, {val: i32} => le(val)
}

test8_be a,  0x0f ; = 0x0f
test8_be a, -0x0f ; = 0xf1

test32_be a,  0x0f ; = 0x00_00_00_0f
test32_be a, -0x0f ; = 0xff_ff_ff_f1

test8_le a,  0x0f ; = 0x0f
test8_le a, -0x0f ; = 0xf1

test32_le a,  0x0f ; = 0x0f_00_00_00
test32_le a, -0x0f ; = 0xf1_ff_ff_ff