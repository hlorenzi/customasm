#subruledef reg
{
    r0 => 0x0
}

#ruledef
{
    movh {rd: reg}, {imm: i16} =>
        0xff @ rd @ 0x0 @ imm

    mov1 %{rd: reg}, {imm: i16} => asm {
        movh {rd}, {imm}
    }

    mov2 % {rd: reg}, {imm: i16} => asm {
        movh {rd}, {imm}
    }

    mov3 %{rd: reg}, ${imm: i16} => asm {
        movh {rd}, {imm}
    }

    mov4 % {rd: reg}, ${imm: i16} => asm {
        movh {rd}, {imm}
    }
}

mov1 %r0, 0x1234  ; = 0xff001234
mov2 % r0, 0x1234 ; = 0xff001234
mov3 %r0, $0x1234  ; = 0xff001234
mov4 % r0, $0x1234 ; = 0xff001234