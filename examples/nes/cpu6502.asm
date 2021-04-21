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
	adc #{imm:   i8 }      => 0x69 @ imm
	adc <{zaddr: u8 }      => 0x65 @ zaddr
	adc <{zaddr: u8 },  x  => 0x75 @ zaddr
	adc  {addr:  u16}      => 0x6d @ addr[7:0] @ addr[15:8]
	adc  {addr:  u16},  x  => 0x7d @ addr[7:0] @ addr[15:8]
	adc  {addr:  u16},  y  => 0x79 @ addr[7:0] @ addr[15:8]
	adc ({zaddr: u8 },  x) => 0x61 @ zaddr
	adc ({zaddr: u8 }), y  => 0x71 @ zaddr

	and #{imm:   i8 }	   => 0x29 @   imm
	and <{zaddr: u8 }      => 0x25 @ zaddr
	and <{zaddr: u8 },  x  => 0x35 @ zaddr
	and  {addr:  u16}	   => 0x2d @  addr[7:0] @ addr[15:8]
	and  {addr:  u16},  x  => 0x3d @  addr[7:0] @ addr[15:8]
	and  {addr:  u16},  y  => 0x39 @  addr[7:0] @ addr[15:8]
	and ({zaddr: u8 },  x) => 0x21 @ zaddr
	and ({zaddr: u8 }), y  => 0x31 @ zaddr

	asl  a               => 0x0a
	asl <{zaddr: u8 }    => 0x07 @ zaddr
	asl <{zaddr: u8 }, x => 0x16 @ zaddr
	asl  {addr:  u16}    => 0x0e @  addr[7:0] @ addr[15:8]
	asl  {addr:  u16}, x => 0x1e @  addr[7:0] @ addr[15:8]

	bcc {addr: cpu6502_reladdr} => 0x90 @ addr
	bcs {addr: cpu6502_reladdr} => 0x80 @ addr
	beq {addr: cpu6502_reladdr} => 0xf0 @ addr

	bit <{zaddr: u8 } => 0x24 @ zaddr
	bit  {addr:  u16} => 0x2C @  addr[7:0] @ addr[15:8]

	bmi {addr: cpu6502_reladdr} => 0x30 @ addr
	bne {addr: cpu6502_reladdr} => 0xd0 @ addr
	bpl {addr: cpu6502_reladdr} => 0x10 @ addr

	brk => 0x00

	bvc {addr: cpu6502_reladdr} => 0x50 @ addr
	bvs {addr: cpu6502_reladdr} => 0x70 @ addr

	clc => 0x18
	cld => 0xd8
	cli => 0x58
	clv => 0xb8

	cmp #{imm:   i8 }      => 0xc9 @   imm
	cmp <{zaddr: u8 }      => 0xc5 @ zaddr
	cmp <{zaddr: u8 },  x  => 0xd5 @ zaddr
	cmp  {addr:  u16}      => 0xcd @  addr[7:0] @ addr[15:8]
	cmp  {addr:  u16},  x  => 0xdd @  addr[7:0] @ addr[15:8]
	cmp  {addr:  u16},  y  => 0xd9 @  addr[7:0] @ addr[15:8]
	cmp ({zaddr: u8 },  x) => 0xc1 @ zaddr
	cmp ({zaddr: u8 }), y  => 0xd1 @ zaddr

	cpx #{imm:   i8 } => 0xe0 @   imm
	cpx <{zaddr: u8 } => 0xe4 @ zaddr
	cpx  {addr:  u16} => 0xec @  addr[7:0] @ addr[15:8]

	cpy #{imm:   i8 } => 0xc0 @   imm
	cpy <{zaddr: u8 } => 0xc4 @ zaddr
	cpy  {addr:  u16} => 0xcc @  addr[7:0] @ addr[15:8]

	dec <{zaddr: u8 }    => 0xc6 @ zaddr
	dec <{zaddr: u8 }, x => 0xd6 @ zaddr
	dec  {addr:  u16}    => 0xce @  addr[7:0] @ addr[15:8]
	dec  {addr:  u16}, x => 0xde @  addr[7:0] @ addr[15:8]

	dex => 0xca
	dey => 0x88

	eor #{imm:   i8 }      => 0x49 @   imm
	eor <{zaddr: u8 }      => 0x45 @ zaddr
	eor <{zaddr: u8 },  x  => 0x55 @ zaddr
	eor  {addr:  u16}      => 0x4d @  addr[7:0] @ addr[15:8]
	eor  {addr:  u16},  x  => 0x5d @  addr[7:0] @ addr[15:8]
	eor  {addr:  u16},  y  => 0x59 @  addr[7:0] @ addr[15:8]
	eor ({zaddr: u8 },  x) => 0x41 @ zaddr
	eor ({zaddr: u8 }), y  => 0x51 @ zaddr

	inc <{zaddr: u8 }    => 0xe6 @ zaddr
	inc <{zaddr: u8 }, x => 0xf6 @ zaddr
	inc  {addr:  u16}    => 0xee @  addr[7:0] @ addr[15:8]
	inc  {addr:  u16}, x => 0xfe @  addr[7:0] @ addr[15:8]

	inx => 0xe8
	iny => 0xc8

	jmp  {addr: u16}  => 0x4c @ addr[7:0] @ addr[15:8]
	jmp ({addr: u16}) => 0x6c @ addr[7:0] @ addr[15:8]

	jsr {addr: u16}  => 0x20 @ addr[7:0] @ addr[15:8]

	lda #{imm:   i8 }      => 0xa9 @   imm
	lda <{zaddr: u8 }      => 0xa5 @ zaddr
	lda <{zaddr: u8 },  x  => 0xb5 @ zaddr
	lda  {addr:  u16}      => 0xad @  addr[7:0] @ addr[15:8]
	lda  {addr:  u16},  x  => 0xbd @  addr[7:0] @ addr[15:8]
	lda  {addr:  u16},  y  => 0xb9 @  addr[7:0] @ addr[15:8]
	lda ({zaddr: u8 },  x) => 0xa1 @ zaddr
	lda ({zaddr: u8 }), y  => 0xb1 @ zaddr

    ldx #{imm:   i8 }    => 0xa2 @   imm
	ldx <{zaddr: u8 }    => 0xa6 @ zaddr
	ldx <{zaddr: u8 }, y => 0xb6 @ zaddr
	ldx  {addr:  u16}    => 0xae @  addr[7:0] @ addr[15:8]
	ldx  {addr:  u16}, y => 0xbe @  addr[7:0] @ addr[15:8]

	ldy #{imm:   i8 }    => 0xa0 @   imm
	ldy <{zaddr: u8 }    => 0xa4 @ zaddr
	ldy <{zaddr: u8 }, x => 0xb4 @ zaddr
	ldy  {addr:  u16}    => 0xac @  addr[7:0] @ addr[15:8]
	ldy  {addr:  u16}, x => 0xbc @  addr[7:0] @ addr[15:8]

	lsr  a               => 0x4a
	lsr <{zaddr: u8 }    => 0x46 @ zaddr
	lsr <{zaddr: u8 }, x => 0x56 @ zaddr
	lsr  {addr:  u16}    => 0x4e @  addr[7:0] @ addr[15:8]
	lsr  {addr:  u16}, x => 0x5e @  addr[7:0] @ addr[15:8]

	nop => 0xea

	ora #{imm:   i8 }      => 0x09 @   imm
	ora <{zaddr: u8 }      => 0x05 @ zaddr
	ora <{zaddr: u8 },  x  => 0x15 @ zaddr
	ora  {addr:  u16}      => 0x0d @  addr[7:0] @ addr[15:8]
	ora  {addr:  u16},  x  => 0x1d @  addr[7:0] @ addr[15:8]
	ora  {addr:  u16},  y  => 0x19 @  addr[7:0] @ addr[15:8]
	ora ({zaddr: u8 },  x) => 0x01 @ zaddr
	ora ({zaddr: u8 }), y  => 0x11 @ zaddr

	pha => 0x48
	php => 0x08
	pla => 0x68
	plp => 0x28

	rol  a               => 0x2a
	rol <{zaddr: u8 }    => 0x26 @ zaddr
	rol <{zaddr: u8 }, x => 0x36 @ zaddr
	rol  {addr:  u16}    => 0x2e @  addr[7:0] @ addr[15:8]
	rol  {addr:  u16}, x => 0x3e @  addr[7:0] @ addr[15:8]
 
	ror  a               => 0x6a
	ror <{zaddr: u8 }    => 0x66 @ zaddr
	ror <{zaddr: u8 }, x => 0x76 @ zaddr
	ror  {addr:  u16}    => 0x6e @  addr[7:0] @ addr[15:8]
	ror  {addr:  u16}, x => 0x7e @  addr[7:0] @ addr[15:8]

	rti => 0x40
	rts => 0x60

	sbc #{imm:   i8 }      => 0xe9 @   imm
	sbc <{zaddr: u8 }      => 0xe5 @ zaddr
	sbc <{zaddr: u8 },  x  => 0xf5 @ zaddr
	sbc  {addr:  u16}      => 0xed @  addr[7:0] @ addr[15:8]
	sbc  {addr:  u16},  x  => 0xfd @  addr[7:0] @ addr[15:8]
	sbc  {addr:  u16},  y  => 0xf9 @  addr[7:0] @ addr[15:8]
	sbc ({zaddr: u8 },  x) => 0xe1 @ zaddr
	sbc ({zaddr: u8 }), y  => 0xf1 @ zaddr

	sec => 0x38
	sed => 0xf8
	sei => 0x78

	sta <{zaddr: u8 }      => 0x85 @ zaddr
	sta <{zaddr: u8 },  x  => 0x95 @ zaddr
	sta  {addr:  u16}      => 0x8d @  addr[7:0] @ addr[15:8]
	sta  {addr:  u16},  x  => 0x9d @  addr[7:0] @ addr[15:8]
	sta  {addr:  u16},  y  => 0x99 @  addr[7:0] @ addr[15:8]
	sta ({zaddr: u8 },  x) => 0x81 @ zaddr
	sta ({zaddr: u8 }), y  => 0x91 @ zaddr

	stx <{zaddr: u8 }    => 0x86 @ zaddr
	stx <{zaddr: u8 }, y => 0x96 @ zaddr
	stx  {addr:  u16}    => 0x8e @  addr[7:0] @ addr[15:8]

	sty <{zaddr: u8 }    => 0x84 @ zaddr
	sty <{zaddr: u8 }, x => 0x94 @ zaddr
	sty  {addr:  u16}    => 0x8c @  addr[7:0] @ addr[15:8]

	tax => 0xaa
	tay => 0xa8
	tsx => 0xba
	txa => 0x8a
	txs => 0x9a
	tya => 0x98
}