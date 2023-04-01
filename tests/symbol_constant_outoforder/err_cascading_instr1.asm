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
}

x = y ; error: converge
    ld x ; error: converge
    ld x ; error: converge
label: ; error: converge
y = label + 3 ; error: converge