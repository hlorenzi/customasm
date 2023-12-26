#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10, "your custom message!")
        0x55 @ x`8
    }
}

ld 0x5 ; = 0x5505
ld -0x20 ; = 0x55e0