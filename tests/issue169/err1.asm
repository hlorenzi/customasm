#subruledef subtest
{
	{x: s32} =>
	{
		assert(x > 0)
		assert(x < 10)
		x
	}
}
#ruledef test
{
	test {s: s32} => s
	testasm {a: subtest} => asm{test {a}}
}

testasm 121 ; error: failed / note:_:13: within / note:_:3: within / error:_:6: assertion