#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
#align 8
ld $ ; = 0x5502