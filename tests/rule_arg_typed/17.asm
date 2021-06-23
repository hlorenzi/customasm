#ruledef test
{
    ld {x: s8} => 0x55 @ x
}

ld 12 ; = 0x550c