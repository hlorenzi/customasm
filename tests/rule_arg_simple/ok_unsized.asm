#ruledef test
{
    ld {x} => 0x55 @ x
}

ld 0x11 ; = 0x5511
ld 0x1122 ; = 0x551122
ld 0x112233 ; = 0x55112233