#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10)
        0x55 @ x`8
    }
}

ld var ; error: failed / note:_:3: within / error:_:5: assertion
var = 0x15