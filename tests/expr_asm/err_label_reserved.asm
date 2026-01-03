#ruledef
{
    jmp {addr: u8} => 0xee @ addr
    loop => asm {
        jmp 0x11
        $incbin:
        jmp 0x22
        jmp $incbin
        jmp 0x33
    }
}

loop ; error: failed / note:_:4: within / error:_:6: reserved
; legacy: off