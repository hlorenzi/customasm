#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
#res 2 ; = 0x0000
#res 3 ; = 0x000000
ld $ ; = 0x5507