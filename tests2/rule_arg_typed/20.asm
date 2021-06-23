#ruledef test
{
    ld {x: s8} => 0x55 @ x
}

ld 0x001 ; = 0x5501