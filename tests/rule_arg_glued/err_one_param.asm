#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

ld r0xff ; error: no match
ld r0x123 ; error: no match
ld 0 ; error: no match