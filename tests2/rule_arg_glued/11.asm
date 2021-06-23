#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

ld r 0 , 0x34 ; = 0x550034