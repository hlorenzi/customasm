#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
#align 16 + 16 ; = 0x0000
ld $ ; = 0x5504