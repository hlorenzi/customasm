#ruledef test
{
    ld {x: u8} =>
    {
        0x11 @ x`24
    }

    ld {x: u16} =>
    {
        0x22 @ x`16
    }

    ld {x: u24} =>
    {
        0x33 @ x`8
    }
}

ld 0x5 ; = 0x3305
ld 0x15 ; = 0x3315
ld 0x215 ; = 0x3315
ld 0x43215 ; = 0x3315