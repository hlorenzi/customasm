#ruledef test
{
    ld {x} => 0x55 @ x`8
}

ld 3 + 4 * 5 ; = 0x5517