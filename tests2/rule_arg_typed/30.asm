#ruledef test
{
    ld {x: s8} => 0x55 @ x
}

ld !0x80 ; error: out of range