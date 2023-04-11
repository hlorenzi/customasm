#ruledef test
{
    ld1 {x: i1} => 0x55 @ 0b000 @ x
}

ld1 0 ; = 0x550
ld1 1 ; = 0x551
ld1 -1 ; = 0x551