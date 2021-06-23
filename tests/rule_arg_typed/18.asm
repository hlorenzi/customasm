#ruledef test
{
    ld {x: s8} => 0x55 @ x
}

ld 6 * 2 ; = 0x550c