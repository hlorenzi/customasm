#ruledef test
{
    ld {x: u8} =>
    {
        0x11 @ x`8
    }

    ld {x: u16} =>
    {
        0x22 @ x`16
    }

    ld {x: u24} =>
    {
        0x33 @ x`24
    }
}

ld 0x5 ; = 0x1105
ld 0x15 ; = 0x1115
ld 0x215 ; = 0x220215
ld 0x43215 ; = 0x33043215