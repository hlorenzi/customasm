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
    }

    op => asm {
        jmp $
        asmjmp $
        jmp $
    }
}

op ; = 0x012
op ; = 0x345