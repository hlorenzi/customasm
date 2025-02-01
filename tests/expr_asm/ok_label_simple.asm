#ruledef
{
    jmp {addr: u8} => 0xee @ addr
    loop => asm {
        jmp 0x11
        label:
        jmp 0x22
        jmp label
        jmp 0x33
    }
}

loop ; = 0xee11_ee22_ee02_ee33