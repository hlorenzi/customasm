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
    
#assert $ == 0x0

    ld label
    ld label
label:

#assert label == 0x4 ; error: assertion

    ld label
    ld label
    
#assert $ == 0x8 ; error: assertion