#subruledef sm83_reladdr {
    {addr: u16} => {
        reladdr = addr - $ - 2
        assert(reladdr <=  0x7f)
		assert(reladdr >= !0x7f)
		reladdr`8
    }
}

; instruction decoding on the SM83 chip uses a weird blend of octal masks and
; binary ones; this is incorporated here using concatenation and slicing to
; "construct" the correct encoding from these fields, with notes regarding
; usage below

; 8-bit registers (plus [hl], which is not a register but instead an indirect
; access to the address stored in the 16-bit register HL described below) are
; B/C/D/E/H/L/[HL]/A, stored as an octal digit/3 bit binary number
#subruledef sm83_r8 {
    B => 0`3
    C => 1`3
    D => 2`3
    E => 3`3
    H => 4`3
    L => 5`3
    [HL] => 6`3
    A => 7`3
}

; 16-bit registers are BC (glue B and C together), DE (ditto D and E), HL
; (ditto H and L), and SP (the stack pointer). There's also AF (A and processor
; flags), which can be used in push/pop instructions but not general ones
#subruledef sm83_r16 {
    BC => 0`2
    DE => 1`2
    HL => 2`2
    SP => 3`2
    AF => assert(0!=0, "invalid operand AF for this instruction")
}

#subruledef sm83_r16_pushpop {
    BC => 0`2
    DE => 1`2
    HL => 2`2
    AF => 3`2
    SP => assert(0!=0, "invalid operand SP for this instruction")
}

; indirect registers are only used in LD [BC/DE/HLI/HLD], A and the inverse
; but we'll put them out here for good measure
#subruledef sm83_r16_indirect {
    [BC] => 0`2
    [DE] => 1`2
    [HLI] => 2`2
    [HL+] => 2`2 ; alternate way to say HLI
    [HLD] => 3`2
    [HL-] => 3`2 ; ditto HLD
}

; conditions are always ordered NZ, Z, NC, C
#subruledef sm83_cond {
    NZ => 0`2
    Z => 1`2
    NC => 2`2
    C => 3`2
}

; ALU operations are an octal field
#subruledef sm83_aluop {
    ADD => 0`3
    ADC => 1`3
    SUB => 2`3
    SBC => 3`3
    AND => 4`3
    XOR => 5`3
    OR  => 6`3
    CP  => 7`3
}

; RST vectors are every 8 bytes before $0040 (which is itself vblank)
#subruledef sm83_rst {
    {n: u8} => {
        assert((n%8==0)&&(n<0x40),"invalid RST vector!")
        n[5:3]
    }
}

; LDH takes an unsigned 8 bit address relative to 0xff00
; allow providing either the byte or 0xff(the byte)
#subruledef sm83_a8 {
    {byte: u8} => byte
    {addr: u16} => {
        assert(addr>=0xff00, "invalid operand address for LDH")
        addr`8
    }
}

; the first 64 prefixed operations are in the pattern 0b00xxxyyy, where xxx is
; an operation in the list RLC/RRC/RL/RR/SLA/SRA/SWAP/SRL, and yyy is an 8-bit
; register; the xxx comes from here
#subruledef sm83_cbprefixop {
    RLC => 0`3
    RRC => 1`3
    RL => 2`3
    RR => 3`3
    SLA => 4`3
    SRA => 5`3
    SWAP => 6`3
    SRL => 7`3
}

; the remaining 192 operations are in the form 0bxxyyyzzz, where xx is one of
; BIT/RES/SET (01/10/11 resp.), yyy is an immediate octal digit, and zzz is an
; 8-bit register; the yyy comes from here
#subruledef sm83_u3 {
    {n} => {
        assert(n<8, "invalid operand (must be in 0-7 range)")
        n`3
    }
}

#subruledef sm83_cbprefix {
    {op: sm83_cbprefixop} {r8: sm83_r8} => 0b00 @ op @ r8
    BIT {n: sm83_u3},{r8: sm83_r8} => 0b01 @ n @ r8
    RES {n: sm83_u3},{r8: sm83_r8} => 0b10 @ n @ r8
    SET {n: sm83_u3},{r8: sm83_r8} => 0b11 @ n @ r8
}

#ruledef sm83 {
    NOP => 0x00
    ; the 16-bit registers in these instructions use the upper 2 bits of the
    ; octal digit as the register index, with the lower bit of the octal digit
    ; being 0 for one set of instructions and 1 for the other
	LD {r16: sm83_r16},{n16: u16} => (r16 @ 0b0 @ 0o1)`8 @ le(n16)
    ADD HL,{r16: sm83_r16} => (r16 @ 0b1 @ 0o1)`8 @ le(n16)
    LD {r16: sm83_r16_indirect},A => (r16 @ 0b0 @ 0o2)`8
    LD A,{r16: sm83_r16_indirect} => (r16 @ 0b1 @ 0o2)`8
    INC {r16: sm83_r16} => (r16 @ 0b0 @ 0o3)`8
    DEC {r16: sm83_r16} => (r16 @ 0b1 @ 0o3)`8
    ; 8-bit register instructions often line up nicely when looked at octally
    ; for example:
    INC {r8: sm83_r8} => (r8 @ 0o4)`8
    DEC {r8: sm83_r8} => (r8 @ 0o5)`8
    LD {r8: sm83_r8},{n8: u8} => (r8 @ 0o6)`8 @ n8
    ; now for the rest of the instructions <0x40
    RLCA => 0o07`8
    RRCA => 0o17`8
    RLA => 0o27`8
    RRA => 0o37`8
    LD [{a16: u16}],SP => 0o10`8 @ le(a16)
    STOP {n8: u8} => 0o20`8 @ n8
    ; alias to let you STOP without defining the following byte
    ; (it's not used, but canonically 0x10 0x00 is the "correct" use of STOP)
    STOP => asm { STOP 0 }
    JR {e8: sm83_reladdr} => 0o30`8 @ e8
    DAA => 0o47
    CPL => 0o57
    SCF => 0o67
    CCF => 0o77
    ; conditional JR uses the lower 2 bits of the octal digit as the condition
    ; code, with the upper bit of the octal digit fixed at 1
    JR {cond: sm83_cond},{e8: sm83_reladdr} => (0b1 @ cond @ 0o0)`8 @ e8
    ; now for load instructions
    ; load instructions are 0b01xxxyyy, where xxx is the destination register
    ; and yyy is the source register
    ; if they are the same, this is functionally a no-op, except in the case
    ; where source and destination are [hl], which would read a byte from memory 
    ; and then write it directly back but instead is the encoding for `halt`
    LD {dst: sm83_r8},{src: sm83_r8} => {
        assert(!(dst==6 && src==6),"`ld [hl],[hl]` encodes as `halt`")
        0b01 @ dst @ src
    }
    HALT => 0x76
    ; ALU operations are 0b10xxxyyy, where xxx is the operation and yyy is the
    ; source register (all ALU operations are on the A register), or 0b11zzz110
    ; where zzz is the operation and the source is an immediate 8-bit value
    {op: sm83_aluop} A,{r8: sm83_r8} => 0b10 @ op @ r8
    {op: sm83_aluop} A,{n8: u8} => 0b10 @ op @ 0o6 @ n8
    ; since all ALU operations are on A, let users omit A
    {op: sm83_aluop} {r8: sm83_r8} => 0b10 @ op @ r8
    {op: sm83_aluop} {n8: u8} => 0b10 @ op @ 0o6 @ n8
    ; now for the rest of 0b11xxxxxx
    RET {cond: sm83_cond} => 0b11 @ 0b0 @ cond @ 0o0
    POP {r16: sm83_r16_pushpop} => 0b11 @ r16 @ 0b0 @ 0o1
    JP {cond: sm83_cond},{a16: u16} => 0b11 @ 0b0 @ cond @ 0o2 @ le(a16)
    JP {a16: u16} => 0xc3 @ le(a16)
    CALL {cond: sm83_cond},{a16: u16} => 0b11 @ 0b0 @ cond @ 0o4 @ le(a16)
    PUSH {r16: sm83_r16_pushpop} => 0b11 @ r16 @ 0b0 @ 0o5
    RST {vector: sm83_rst} => 0b11 @ vector @ 0o7
    RET => 0xc9
    {instr: sm83_cbprefix} => 0xcb @ instr
    CALL {a16: u16} => 0xcd @ le(a16)
    RETI => 0xd9
    LDH [{a8: sm83_a8}],A => 0xe0 @ a8
    LDH [C],A => 0xe2
    ADD SP, {e8: s8} => 0xe8 @ e8
    JP HL => 0xe9
    LD [{a16: u16}],A => 0xea @ le(a16)
    LDH A,[{a8: sm83_a8}] => 0xf0 @ a8
    LDH A,[C] => 0xf2
    DI => 0xf3
    LD HL,SP+{e8: s8} => 0xf8 @ e8
    LD SP, HL => 0xf9
    LD A,[{a16: u16}] => 0xfa @ le(a16)
    EI => 0xfb
}