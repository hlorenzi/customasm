#ruledef test
{
    ld {x} => 0x55 @ x`8
}


#labelalign 32
ld $ ; = 0x5500
label: ; = 0x0000
ld $ ; = 0x5504
.sublabel:
ld $ ; = 0x5506