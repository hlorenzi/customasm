#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

ld r257, 0x102 ; = 0x550102