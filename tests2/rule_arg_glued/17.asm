#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

x = 0
ld rx, x ; error: no match