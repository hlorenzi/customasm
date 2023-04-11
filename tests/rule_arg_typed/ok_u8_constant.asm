#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

x = 0xff
ld x ; = 0x55ff