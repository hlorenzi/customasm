#ruledef test
{
    ld r{x}, {y} => 0x55 @ x`8 @ y`8
}

ld 0, 0x12 ; error: no match