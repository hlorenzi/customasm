#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10)
        0x55 @ x`8
    }
}

ld 0x15 ; error: failed to resolve