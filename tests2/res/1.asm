#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
#res 0
ld $ ; = 0x5502