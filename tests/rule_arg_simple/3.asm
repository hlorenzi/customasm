#ruledef test
{
    ld {x} => 0x55 @ x[7:0]
}

ld 0xff ; = 0x55ff