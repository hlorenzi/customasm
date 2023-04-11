#subruledef Reg
{
    x => 0x00
    y => 0xff
}

#ruledef
{
    add {r: Reg} =>
	{
		assert(r != 0xff)
		r
	}
}

add x
add y ; error: failed / note:_:9: within / error:_:11: assertion