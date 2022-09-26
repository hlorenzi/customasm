#subruledef register
{
	j  => 0x0a
	ij => 0x0b
}

#ruledef
{
    ld  {r: register}, {addr:  u8} => 0x01 @ r @ addr
    ldi {r: register}, {value: u8} => 0x02 @ r @ value
}

ld  j,  0x55 ; = 0x010a55
ldi j,  0x55 ; = 0x020a55
ld  ij, 0x55 ; = 0x010b55
ldi ij, 0x55 ; = 0x020b55