#subruledef Reg
{
    x => 0x00
    y => 0xff
}

#ruledef
{
    add {r: Reg} =>
	{
		assert((r & 0xff) != 0xff)
		r
	}
}

add x
add y
; error:_:17: failed to resolve
; error:_:11: assertion