#ruledef test
{
    ld {x: i8} => 0x55 @ x
}

ld 0xff ; = 0x55ff