#ruledef test
{
    ld {x: u8} =>
    {
        0x11 @ x`24
    }

    ld {x: u16} =>
    {
        0x22 @ x`24
    }

    ld {x: u24} =>
    {
        0x33 @ x`24
    }
}

ld 0x15 ; error: failed / error: multiple / note:_:3: candidate / note:_:8: candidate / note:_:13: candidate