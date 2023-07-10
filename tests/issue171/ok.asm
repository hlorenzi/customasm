X=1
#if X==1
{
    #subruledef test
	{
		{a: i32} => a
	}
}
#else
{
	#subruledef test
	{
		{a: i16} => a
	}
}

#ruledef
{
	test {X: test} => X
}

test 0x12345678 ; = 0x12345678