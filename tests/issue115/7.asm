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
        ld globalLabel
        ld globalLabel
        ld globalLabel
    }
}

test ; error: converge
globalLabel: