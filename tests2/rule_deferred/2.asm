#ruledef test
{
    ld {x} => 0x55 @ x`8
}

ld var ; = 0x5511
var = 0x11