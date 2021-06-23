#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld 12 ; = 0x550c