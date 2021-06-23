#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

ld r0xff ; error: no match