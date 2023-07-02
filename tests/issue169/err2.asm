#subruledef cpu6502_reladdr
{
	{addr: u16} =>
	{
		reladdr = addr - $ - 2
		assert(reladdr <=  0x7f)
		assert(reladdr >= !0x7f)
		reladdr`8
	}
}

#ruledef cpu6502
{
	bcc {addr: cpu6502_reladdr} => 0x90 @ addr
}

#ruledef cpu6502_macro
{
	bccmacro {addr: cpu6502_reladdr} => asm{bcc {addr}}
}

bcc 120
bccmacro 16384 ; error: failed / note:_:19: within / note:_:3: within / error:_:6: assertion
bcc 120