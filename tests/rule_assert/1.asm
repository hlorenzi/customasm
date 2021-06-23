#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10)
        0x55 @ x`8
    }
}

ld 0x5 ; = 0x5505