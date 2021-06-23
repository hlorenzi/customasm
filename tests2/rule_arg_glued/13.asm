#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

ld r 0xff, 0x12 ; = 0x55ff12