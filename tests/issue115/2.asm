#bits 4

#ruledef {
    jmp {addr:u4} => addr
    asmjmp {addr:u4} => asm{
        jmp addr
        jmp $
    }
    op => asm {
        jmp $
        asmjmp $
        jmp $
    }
}

op  ; = 0x0123
#d asm { op
op} ; = 0x456789ab
op  ; = 0xcdef