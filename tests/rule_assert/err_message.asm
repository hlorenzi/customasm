#ruledef test
{
    ld {x} =>
    {
        assert(x < 0x10, "your custom message!")
        0x55 @ x`8
    }
}

ld 0x15 ; error: failed / note:_:3: within / error:_:5: your custom message!