#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
#align 24 ; = 0x00
#align 32 ; = 0x00
ld $ ; = 0x5504