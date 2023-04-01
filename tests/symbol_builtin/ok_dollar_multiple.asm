#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
ld $ ; = 0x5502
ld $ ; = 0x5504
ld $ ; = 0x5506