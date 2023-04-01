#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
#res 1 + 2 ; = 0x000000
ld $ ; = 0x5505