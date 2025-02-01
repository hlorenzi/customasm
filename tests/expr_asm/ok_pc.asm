#ruledef
{
    ld {addr: u8} => 0xee @ addr
    test => asm {
        ld 0x11
        ld 0x22
        ld $
        ld 0x33
        ld $
        ld 0x44
    }
}

test ; = 0xee11_ee22_ee04_ee33_ee08_ee44