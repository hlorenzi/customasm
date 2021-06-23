#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld x ; = 0x55ff
x = 0xff