#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

ld 0 ; error: no match