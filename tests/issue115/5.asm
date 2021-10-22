#ruledef test
{
    ld {x} =>
    {
        assert(x <= 0x8)
        0x11 @ x`16
    }
 
    ld {x} =>
    {
        assert(x > 0x8)
        0x22 @ x`8
    }

    test => asm
    {
        label:
        ld label
        ld label
        ld label
    }
}

test ; = 0x110000_110000_110000