#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld 255 ; = 0x55ff