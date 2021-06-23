#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

ld r0x123, 0x12 ; error: no match