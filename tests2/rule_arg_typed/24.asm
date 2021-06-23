#ruledef test
{
    ld {x: s8} => 0x55 @ x
}

ld -0x1 ; = 0x55ff