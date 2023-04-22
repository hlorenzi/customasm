#bankdef a
{
    bits = 4
    outp = 0
}

#ruledef
{
    jmp {addr: u4} => addr

    asmjmp {addr: u4} => asm {
        jmp {addr}
        jmp $
    }

    op => asm {
        jmp $
        asmjmp $
        jmp $
    }
}

op  ; = 0x0123
#d asm {
    op ; = 0x4567
    op ; = 0x89ab
}
op  ; = 0xcdef