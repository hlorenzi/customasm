#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

x = 0x12
ld rx, x ; = 0x551212
ld r x + 6, x ; = 0x551812
ld r(x + 6), x ; = 0x551812
ld rx + 6, x ; = 0x551812