#subruledef inner
{
    a   => 0x11
    b   => 0x22
    c.w => 0x33
}

#ruledef test
{
    ld {reg: inner}, {x} => 0x55 @ reg`8 @ x`8
}

ld c, 0xaa ; error: no match