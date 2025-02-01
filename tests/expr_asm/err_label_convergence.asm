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
    
    test => asm {
        ld label
        ld label
        ld label
        label:
    }
}

test ; error: failed / note:_:15: within / error:_:15: did not converge
