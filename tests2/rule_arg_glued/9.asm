#ruledef test
{
    ld r{x} => 0x55 @ x`8
}

x = 0
ld rx ; error: no match