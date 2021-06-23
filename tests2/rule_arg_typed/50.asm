#ruledef test
{
    ld1 {x: i1} => 0x55 @ 0b000 @ x
}

ld1 -2 ; error: out of range