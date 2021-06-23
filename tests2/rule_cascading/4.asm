#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10)
        0x110 @ x`4
    }

    ld {x} =>
    {
        assert(x >= 0x10 && x < 0x100)
        0x22 @ x`8
    }

    ld {x} =>
    {
        assert(x >= 0x100)
        0x33 @ x`16
    }
}

ld 0x43215 ; = 0x333215