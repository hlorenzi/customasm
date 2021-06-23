#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld -1 ; = 0x55ff