#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

x = 0x12
ld rx ; = 0x5512
ld r x + 6 ; = 0x5518
ld r(x + 6) ; = 0x5518
ld rx + 6 ; = 0x5518