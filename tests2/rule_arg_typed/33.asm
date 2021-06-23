#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld 6 * 2 ; = 0x550c