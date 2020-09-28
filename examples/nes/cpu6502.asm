#ruledef cpu6502
{
	sei => 0x78
	cld => 0xd8
	
	rti => 0x40

	lda #{imm}    => 0xa9 @  imm[7:0]
	lda {addr}    => 0xad @ addr[7:0] @ addr[15:8]
	lda {addr}, x => 0xbd @ addr[7:0] @ addr[15:8]
	ldx #{imm}    => 0xa2 @  imm[7:0]
	ldx {addr}    => 0xae @ addr[7:0] @ addr[15:8]
                     
	sta {addr}    => 0x8d @ addr[7:0] @ addr[15:8]
	sta {addr}, x => 0x9d @ addr[7:0] @ addr[15:8]
	stx {addr}    => 0x8e @ addr[7:0] @ addr[15:8]

	inc {addr} => 0xee @ addr[7:0] @ addr[15:8]
	inx        => 0xe8

	dec {addr} => 0xce @ addr[7:0] @ addr[15:8]
	
	txs => 0x9a

	bit {addr} => 0x2c @ addr[7:0] @ addr[15:8]

	cmp #{imm} => 0xc9 @ imm[7:0]
	
	jmp {addr} => 0x4c @ addr[7:0] @ addr[15:8]
	
	bne {addr} =>
	{
		reladdr = addr - $ - 2
		assert(reladdr <=  0x7f)
		assert(reladdr >= !0x7f)
		0xd0 @ reladdr[7:0]
	}
	
	bpl {addr} =>
	{
		reladdr = addr - $ - 2
		assert(reladdr <=  0x7f)
		assert(reladdr >= !0x7f)
		0x10 @ reladdr[7:0]
	}
}