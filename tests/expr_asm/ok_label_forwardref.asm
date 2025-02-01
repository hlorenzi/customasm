#ruledef
{
    jmp {addr: u8} => 0xee @ addr
    loop => asm {
        jmp 0x11
        jmp label
        label:
        jmp 0x22
        jmp 0x33
    }
}

loop ; = 0xee11_ee04_ee22_ee33