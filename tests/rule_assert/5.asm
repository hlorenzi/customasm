#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10)
        0x55 @ x`8
    }
}

ld var ; error: failed / error:_:5: assertion
var = 0x15