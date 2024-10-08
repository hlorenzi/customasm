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

    ld label ; = 0x110006
    ld label ; = 0x110006
label:

#assert label == 0x6

    ld label ; = 0x110006
    ld label ; = 0x110006
    
#assert $ == 0xc