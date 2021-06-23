#ruledef test
{
    ld {x: u8} => 0x55 @ x
}

ld 0x001 ; = 0x5501