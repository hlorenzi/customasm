#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld 255 ; = 0x55ff