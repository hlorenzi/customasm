#ruledef test
{
    ld {x} => 0x55 @ x`8
}


ld $ ; = 0x5500
label1:
ld $ ; = 0x5502
#labelalign 64
label2: ; = 0x00000000
ld $ ; = 0x5508